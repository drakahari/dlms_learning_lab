//! Source-bound IPv6 configuration tasks with independent example and assignment state.
use crate::{
    host_workspace::Requirement, linux_filter::Chain, linux_v6_admin as core,
    linux_v6_filter::Verdict, linux_v6_path as path, linux_v6_route_equivalence,
    linux_v6_saved_settings as file, linux_v6_workspace::Check, script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv6Addr};
type Result<T> = std::result::Result<T, String>;
pub const MAX_CONSOLE: usize = 128 * 1024;
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn bounded<T: Serialize>(v: &T, n: usize) -> bool {
    serde_json::to_vec(v).is_ok_and(|b| b.len() <= n)
}
fn check(passed: bool, label: impl Into<String>, detail: impl Into<String>) -> Check {
    Check {
        passed,
        label: label.into(),
        detail: detail.into(),
    }
}
fn route_text(v: &Option<(String, Option<Ipv6Addr>)>) -> String {
    match v {
        None => "no route".into(),
        Some((d, None)) => format!("directly on {d}"),
        Some((d, Some(g))) => format!("via {g} on {d}"),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setting {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub key: String,
    pub value: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub why: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathTarget {
    pub target: Ipv6Addr,
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub device: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub advertisement: core::Advertisement,
    pub accepted: bool,
    pub learns_default: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub ingress: String,
    pub header: path::Header,
    pub expected: path::Gate,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub initial: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,16>")]
    pub requirements: Vec<Requirement>,
    pub expected_routes: path::Facts,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Setting,8>")]
    pub settings: Vec<Setting>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,PathTarget,8>")]
    pub paths: Vec<PathTarget>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Message,4>")]
    pub messages: Vec<Message>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Frame,8>")]
    pub frames: Vec<Frame>,
    #[serde(deserialize_with = "crate::apache_static_workspace::option_text::<_,15>")]
    pub active_learned: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<SettingsFile>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsFile {
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub initial_text: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestartTrial {
    pub world: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Check,64>")]
    pub checks: Vec<Check>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trial {
    pub frame: usize,
    pub trace: Option<path::Trace>,
    pub physical: Option<Check>,
    pub result: Check,
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.expected_routes.validate().map_err(|e| e.message)?;
        if let Some(f) = &self.file {
            file::parse(&self.initial.settings, &f.initial_text)
                .map_err(|e| format!("Invalid supplied settings file: {e}"))?;
        }
        if !id(&self.id)
            || !bounded(self, 256 * 1024)
            || self.requirements.is_empty()
            || self.requirements.len() > 16
            || self.settings.len() > 8
            || self.paths.is_empty()
            || self.paths.len() > 8
            || self.messages.len() > 4
            || self.frames.is_empty()
            || self.frames.len() > 8
        {
            return Err("Supply a stable bounded configuration task with address requirements, physical paths and transit cases.".into());
        }
        let names = self
            .initial
            .settings
            .interfaces
            .iter()
            .map(|i| i.name.as_str())
            .collect::<BTreeSet<_>>();
        if names
            != self
                .expected_routes
                .links
                .iter()
                .map(|l| l.name.as_str())
                .collect()
        {
            return Err("Expected route facts must use the supplied interfaces.".into());
        }
        if self.requirements.iter().any(|r| {
            !matches!(
                r,
                Requirement::Ipv6Address { .. }
                    | Requirement::Up { .. }
                    | Requirement::PreserveIpv4 {}
                    | Requirement::PreserveIpv6Interface { .. }
                    | Requirement::PreserveIpv6Routes { .. }
            )
        }) {
            return Err("Use IPv6 address/preservation conditions here; RA routes and actual paths have their own checks.".into());
        }
        let mut settings = BTreeSet::new();
        for s in &self.settings {
            if s.label.trim().is_empty() || s.why.trim().is_empty() || !settings.insert(&s.key) {
                return Err("Give each required setting one clear explanation.".into());
            }
            let mut probe = self.initial.settings.clone();
            probe
                .command(&format!("sysctl -w {}={}", s.key, s.value))
                .map_err(|e| e.message)?;
        }
        for p in &self.paths {
            if !names.contains(p.device.as_str()) {
                return Err("A path condition names an unknown interface.".into());
            }
        }
        let mut messages = BTreeSet::new();
        for m in &self.messages {
            if !id(&m.id) || m.label.trim().is_empty() || !messages.insert(&m.id) {
                return Err("Message cases need unique stable IDs and readable labels.".into());
            }
            let mut probe = self.initial.clone();
            probe.learned.clear();
            probe.observe_ra(&m.advertisement)?;
        }
        if self
            .active_learned
            .as_ref()
            .is_some_and(|d| !names.contains(d.as_str()))
        {
            return Err("The learned-default goal names an unknown interface.".into());
        }
        let mut frames = BTreeSet::new();
        for f in &self.frames {
            if !id(&f.id)
                || f.label.trim().is_empty()
                || !frames.insert(&f.id)
                || !names.contains(f.ingress.as_str())
                || !matches!(f.expected, path::Gate::Allowed | path::Gate::ScopeBound)
            {
                return Err("Transit cases need unique IDs, known ingress and an allowed-or-link-local outcome.".into());
            }
            let expected = path::Runtime {
                filter: crate::linux_v6_filter::Table::new(Verdict::Accept),
                forwarding: true,
            }
            .evaluate(
                &self.expected_routes,
                &path::Origin::Incoming(f.ingress.clone()),
                &f.header,
            )
            .map_err(|e| e.message)?;
            if expected.gate != f.expected
                || (f.expected == path::Gate::Allowed && expected.chain != Some(Chain::Forward))
            {
                return Err(
                    "Supplied expected routes must reach the declared transit/link-local outcomes."
                        .into(),
                );
            }
        }
        Ok(())
    }
    pub fn trial(&self, s: &core::State, n: usize) -> Result<Trial> {
        let f = self.frames.get(n).ok_or("Unknown transit case.")?;
        let fail = |detail: String| Trial {
            frame: n,
            trace: None,
            physical: None,
            result: check(false, &f.label, detail),
        };
        let facts = match s.facts() {
            Ok(v) => v,
            Err(e) => return Ok(fail(e)),
        };
        let forwarding = s
            .settings
            .forwards_from(&f.ingress)
            .map_err(|e| e.message)?;
        // Configuration/header practice does not claim connection tracking or a
        // server-generated reply. Keep link-local scope evaluation untracked.
        let trace = match (path::Runtime {
            filter: crate::linux_v6_filter::Table::new(Verdict::Accept),
            forwarding,
        })
        .evaluate(
            &facts,
            &path::Origin::Incoming(f.ingress.clone()),
            &f.header,
        ) {
            Ok(v) => v,
            Err(e) => return Ok(fail(e.message)),
        };
        let physical = if trace.gate == path::Gate::Allowed {
            let p = s.supplied_path(f.header.destination)?;
            Some(check(p.pass, "Supplied physical path", p.reason))
        } else {
            None
        };
        let passed = trace.gate == f.expected
            && physical.as_ref().is_none_or(|p| p.passed)
            && (f.expected != path::Gate::Allowed || trace.chain == Some(Chain::Forward));
        let detail = if let Some(p) = physical.as_ref().filter(|p| !p.passed) {
            p.detail.clone()
        } else {
            match trace.gate {
            path::Gate::Allowed=>"The configured route, forwarding and supplied path permit this transit header.".into(),
            path::Gate::ScopeBound=>"Link-local addresses stay on their own link; they are not forwarded to another network.".into(),
            path::Gate::ForwardingDisabled=>"The route exists, but IPv6 forwarding is disabled for this ingress. A route alone does not enable transit.".into(),
            path::Gate::NoRoute=>"No modeled IPv6 route reaches this destination. Inspect the destination prefix, gateway and interface.".into(),
            path::Gate::IngressDown|path::Gate::EgressDown=>"A supplied interface is down; this is not a successful transit test.".into(),
            _=>"The modeled path or filter did not permit this header.".into(),
        }
        };
        Ok(Trial {
            frame: n,
            trace: Some(trace),
            physical,
            result: check(passed, &f.label, detail),
        })
    }
    pub fn checks(&self, s: &core::State) -> Vec<Check> {
        if let Err(e) = s.validate() {
            return vec![check(false, "Router configuration", e)];
        }
        let mut rows = self
            .requirements
            .iter()
            .map(|r| {
                let (p, d) = r.check(&self.initial.host, &s.host);
                let label = match r {
                    Requirement::Ipv6Address { device, .. } => format!("IPv6 address on {device}"),
                    Requirement::Up { device } => format!("Interface {device} available"),
                    Requirement::PreserveIpv4 {} => "Preserve IPv4 configuration".into(),
                    Requirement::PreserveIpv6Interface { device } => {
                        format!("Preserve {device}'s IPv6 address")
                    }
                    Requirement::PreserveIpv6Routes { .. } => {
                        "Preserve the supplied IPv6 routes".into()
                    }
                    _ => "Configuration requirement".into(),
                };
                check(p, label, d)
            })
            .collect::<Vec<_>>();
        for wanted in &self.settings {
            let actual = s.settings.get(&wanted.key);
            let passed = actual.as_ref().is_ok_and(|v| *v == wanted.value);
            rows.push(check(
                passed,
                &wanted.label,
                if passed {
                    wanted.why.clone()
                } else {
                    format!(
                        "{} Expected {} = {}; actual {}.",
                        wanted.why,
                        wanted.key,
                        wanted.value,
                        actual.map_or_else(|e| e.message, |v| v.to_string())
                    )
                },
            ));
        }
        let routes = s
            .facts()
            .and_then(|a| linux_v6_route_equivalence::compare(&a, &self.expected_routes));
        rows.push(match routes {Ok(None)=>check(true,"Complete route plan","All modeled destination regions select the required interface and next hop; equivalent split prefixes and irrelevant metrics are accepted."),Ok(Some(d))=>check(false,"Complete route plan",format!("The route plan differs starting at prefix boundary {}: required {}; actual {}. Inspect broad defaults, missing prefix coverage, device scope and next hop; matching only displayed clients is insufficient.",d.address,route_text(&d.expected),route_text(&d.actual))),Err(e)=>check(false,"Complete route plan",e)});
        for p in &self.paths {
            rows.push(match s.supplied_path(p.target) {
                Ok(v) => check(
                    v.pass && v.selection.as_ref().is_some_and(|v| v.device == p.device),
                    format!("Supplied path to {}", p.target),
                    format!("Required outgoing interface {}. {}", p.device, v.reason),
                ),
                Err(e) => check(false, format!("Supplied path to {}", p.target), e),
            });
        }
        for m in &self.messages {
            let mut probe = s.clone();
            probe.learned.clear();
            let result = probe.observe_ra(&m.advertisement);
            let learned = probe.learned.iter().any(|a| {
                a.device == m.advertisement.device && a.gateway == m.advertisement.gateway
            });
            rows.push(match result {
                Ok(o) => check(
                    o.accepted == m.accepted && learned == m.learns_default,
                    &m.label,
                    format!(
                        "Expected: {} the message, {}. Actual: message {}, {}. {}",
                        if m.accepted { "accept" } else { "ignore" },
                        if m.learns_default {
                            "learn a default"
                        } else {
                            "no learned default"
                        },
                        if o.accepted { "accepted" } else { "ignored" },
                        if learned {
                            "default learned"
                        } else {
                            "no default learned"
                        },
                        o.detail
                    ),
                ),
                Err(e) => check(false, &m.label, e),
            });
        }
        if let Some(device) = &self.active_learned {
            let active = s
                .facts()
                .and_then(|f| f.route(Ipv6Addr::UNSPECIFIED).map_err(|e| e.message));
            let passed = active.as_ref().is_ok_and(|r| {
                r.as_ref().is_some_and(|r| {
                    r.device == *device
                        && r.metric == core::RA_METRIC
                        && s.learned
                            .iter()
                            .any(|a| a.device == r.device && Some(a.gateway) == r.via)
                })
            });
            rows.push(check(passed,"Active learned default",format!("Use an actually observed learned default on {device}. Changing accept_ra does not invent a new advertisement; a lower-metric manual default takes precedence.")));
        }
        for n in 0..self.frames.len() {
            rows.push(
                self.trial(s, n)
                    .map_or_else(|e| check(false, "Transit case", e), |t| t.result),
            );
        }
        rows
    }
    pub fn restart_trial(&self, world: &core::State, saved: &str) -> Result<RestartTrial> {
        if self.file.is_none() {
            return Err("This lesson has no saved-settings restart activity.".into());
        }
        let mut fresh = file::fresh(world, saved).map_err(|e| e.to_string())?;
        for message in self
            .messages
            .iter()
            .filter(|m| m.accepted && m.learns_default)
        {
            fresh.observe_ra(&message.advertisement)?;
        }
        let checks = self.checks(&fresh);
        Ok(RestartTrial {
            world: fresh,
            checks,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum History {
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        input: String,
    },
    Observe {
        message: usize,
    },
    SaveFile {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
}
fn apply_history(
    world: &mut core::State,
    saved: &mut String,
    t: &Task,
    h: &History,
) -> Result<String> {
    match h {
        History::Command { input } => {
            let words = crate::linux_filter::words(input).map_err(|e| e.message)?;
            let args = words.iter().map(String::as_str).collect::<Vec<_>>();
            match args.as_slice() {
                ["sysctl", "-p", path] if t.file.is_some() && *path == file::FILE => {
                    file::apply(world, saved).map_err(|e| format!("Settings not loaded: {e}"))
                }
                ["cat", path] if t.file.is_some() && *path == file::FILE => Ok(saved.clone()),
                _ => world.command(input).map(|o| o.text),
            }
        }
        History::Observe { message } => {
            let m = t
                .messages
                .get(*message)
                .ok_or("Unknown supplied advertisement.")?;
            world
                .observe_ra(&m.advertisement)
                .map(|o| format!("[Simulator RA observation] {}\n", o.detail))
        }
        History::SaveFile { text } => {
            if t.file.is_none() {
                return Err("This lesson has no virtual settings file.".into());
            }
            file::parse(&world.settings, text).map_err(|e| format!("File not saved: {e}"))?;
            *saved = text.clone();
            Ok(
                "Saved the virtual settings file; live settings are unchanged until loaded.\n"
                    .into(),
            )
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub world: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,History,96>")]
    pub history: Vec<History>,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,131072>")]
    pub output: String,
    pub selected_message: usize,
    pub selected_frame: usize,
    pub last_trial: Option<Trial>,
    pub checked: bool,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        deserialize_with = "crate::cron::text::<_,4096>"
    )]
    pub file_editor: String,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        deserialize_with = "crate::cron::text::<_,4096>"
    )]
    pub saved_file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_restart: Option<RestartTrial>,
}
fn log(s: &mut Scope, text: &str) {
    s.output.push_str(text);
    if s.output.len() > MAX_CONSOLE {
        let mut at = s.output.len() - MAX_CONSOLE;
        while !s.output.is_char_boundary(at) {
            at += 1;
        }
        s.output.drain(..at);
    }
}
impl Scope {
    pub fn new(t: &Task) -> Self {
        Self {
            world: t.initial.clone(),
            history: vec![],
            input: String::new(),
            output: String::new(),
            selected_message: 0,
            selected_frame: 0,
            last_trial: None,
            checked: false,
            file_editor: t
                .file
                .as_ref()
                .map_or(String::new(), |f| f.initial_text.clone()),
            saved_file: t
                .file
                .as_ref()
                .map_or(String::new(), |f| f.initial_text.clone()),
            last_restart: None,
        }
    }
    pub fn shape(&self) -> Result<()> {
        self.world.validate()?;
        if !bounded(self, 512 * 1024)
            || self.history.len() > 96
            || self.input.len() > 512
            || self.output.len() > MAX_CONSOLE
            || !file::text_ok(&self.file_editor)
            || !file::text_ok(&self.saved_file)
        {
            return Err("Saved router work exceeds its limits; previous work is retained.".into());
        }
        Ok(())
    }
    pub fn bound(&self, t: &Task) -> Result<()> {
        self.shape()?;
        if self.selected_frame >= t.frames.len() || self.selected_message >= t.messages.len().max(1)
        {
            return Err("Saved selection is not part of this lesson.".into());
        }
        let mut replay = t.initial.clone();
        let mut saved = t
            .file
            .as_ref()
            .map_or(String::new(), |f| f.initial_text.clone());
        for h in &self.history {
            apply_history(&mut replay, &mut saved, t, h)?;
        }
        if replay != self.world
            || saved != self.saved_file
            || (t.file.is_none() && !self.file_editor.is_empty())
        {
            return Err("Saved configuration differs from the supported edits in this attempt; previous work is retained.".into());
        }
        if let Some(trial) = &self.last_trial
            && (trial.frame != self.selected_frame
                || *trial != t.trial(&self.world, trial.frame)?)
        {
            return Err("Saved traffic feedback no longer matches the current configuration; previous work is retained.".into());
        }
        if let Some(trial) = &self.last_restart
            && *trial != t.restart_trial(&self.world, &self.saved_file)?
        {
            return Err("Saved restart feedback differs from current saved settings and the supplied network.".into());
        }
        Ok(())
    }
    pub fn checks(&self, t: &Task) -> Vec<Check> {
        if let Err(e) = self.bound(t) {
            return vec![check(false, "Saved workspace", e)];
        }
        let mut rows = t.checks(&self.world);
        if t.file.is_some() {
            rows.push(check(
                self.file_editor == self.saved_file,
                "Save the current settings file",
                "Editor changes must be saved before they count as startup configuration.",
            ));
            rows.push(match t.restart_trial(&self.world,&self.saved_file) {
                Ok(trial)=>{let failures=trial.checks.iter().filter(|c|!c.passed).collect::<Vec<_>>();check(failures.is_empty(),"Saved settings after restart",failures.first().map_or("Fresh settings loaded the saved file; supplied messages and paths satisfy the task. The supplied network remains already persistent.".into(),|c|format!("Each restart test uses fresh settings and learned routes, then the saved file and supplied message. {}: {}",c.label,c.detail)))},
                Err(e)=>check(false,"Saved settings after restart",e),
            });
        }
        rows
    }
    pub fn apply(&mut self, t: &Task, a: &Action) -> Result<()> {
        self.apply_inner(t, a, false)
    }
    pub fn reference(&mut self, t: &Task, a: &Action) -> Result<()> {
        self.apply_inner(t, a, true)
    }
    fn apply_inner(&mut self, t: &Task, a: &Action, strict: bool) -> Result<()> {
        self.bound(t)?;
        let mut next = self.clone();
        match a {
            Action::Input { text } => {
                if text.len() > 512 {
                    return Err("Keep one command draft within 512 bytes.".into());
                }
                next.input = text.clone();
                next.checked = false;
            }
            Action::SelectMessage { message } => {
                if *message >= t.messages.len() {
                    return Err("Choose a supplied advertisement.".into());
                }
                next.selected_message = *message;
            }
            Action::SelectFrame { frame } => {
                if *frame >= t.frames.len() {
                    return Err("Choose a supplied transit case.".into());
                }
                next.selected_frame = *frame;
                next.last_trial = None;
            }
            Action::RunFrame => {
                next.last_trial = Some(t.trial(&next.world, next.selected_frame)?);
            }
            Action::EditFile { text } => {
                if t.file.is_none() || !file::text_ok(text) {
                    return Err("Edit the supplied virtual settings file within 4 KiB.".into());
                }
                next.file_editor = text.clone();
                next.checked = false;
                next.last_trial = None;
                next.last_restart = None;
            }
            Action::RunRestart => {
                next.last_restart = Some(t.restart_trial(&next.world, &next.saved_file)?);
            }
            Action::Check => {
                next.checked = true;
            }
            Action::Command { .. } | Action::Observe { .. } | Action::SaveFile => {
                if let Action::Command { input } = a
                    && input.len() > 512
                {
                    return Err("Keep one literal command within 512 bytes.".into());
                }
                let h = match a {
                    Action::Command { input } => History::Command {
                        input: input.clone(),
                    },
                    Action::Observe { message } => History::Observe { message: *message },
                    Action::SaveFile => History::SaveFile {
                        text: next.file_editor.clone(),
                    },
                    _ => unreachable!(),
                };
                next.checked = false;
                next.last_trial = None;
                next.last_restart = None;
                if let Action::Command { input } = a {
                    log(&mut next, &format!("$ {input}\n"));
                }
                let mut world = next.world.clone();
                let mut saved = next.saved_file.clone();
                match apply_history(&mut world, &mut saved, t, &h) {
                    Ok(text) => {
                        if world != next.world || saved != next.saved_file {
                            if next.history.len() == 96 {
                                return Err("This attempt has reached 96 configuration changes. Its work is retained; no further change was applied.".into());
                            }
                            next.history.push(h);
                            next.world = world;
                            next.saved_file = saved;
                        }
                        log(&mut next, &text);
                        if matches!(a, Action::Command { .. }) {
                            next.input.clear();
                        }
                    }
                    Err(e) => {
                        if strict {
                            return Err(e);
                        }
                        log(&mut next, &format!("[Not applied] {e}\n"));
                        if let Action::Command { input } = a {
                            next.input = input.clone();
                        }
                    }
                }
            }
        }
        next.bound(t)?;
        *self = next;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Input {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        text: String,
    },
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        input: String,
    },
    Observe {
        message: usize,
    },
    SelectMessage {
        message: usize,
    },
    SelectFrame {
        frame: usize,
    },
    RunFrame,
    Check,
    EditFile {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
    SaveFile,
    RunRestart,
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
    pub fn validate(&self) -> Result<()> {
        self.assignment.validate()?;
        self.example.validate()?;
        if !id(&self.source_id)
            || !bounded(self, 768 * 1024)
            || self.worked.is_empty()
            || self.worked.len() > 32
            || self.assignment.id == self.example.id
            || self.assignment.initial.host.profile.ipv6 == self.example.initial.host.profile.ipv6
        {
            return Err(
                "Use distinct independent/example IPv6 tasks and a stable bounded source.".into(),
            );
        }
        for t in [&self.assignment, &self.example] {
            if Scope::new(t).checks(t).iter().all(|c| c.passed) {
                return Err("A configuration assignment must require a genuine repair.".into());
            }
        }
        let mut example = Scope::new(&self.example);
        for a in &self.worked {
            example.reference(&self.example, a)?;
        }
        if !example.checked || !example.checks(&self.example).iter().all(|c| c.passed) {
            return Err(
                "Worked actions do not reach all example outcomes with an explicit check.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        if s.source_id != self.source_id || !bounded(s, 1024 * 1024) {
            return Err("Saved work belongs to another configuration lesson or exceeds its bound; previous work is retained.".into());
        }
        s.assignment.bound(&self.assignment)?;
        s.practice.bound(&self.example)
    }
    pub fn solved(&self, s: &Session) -> bool {
        self.bound(s).is_ok()
            && s.assignment.checked
            && s.assignment
                .checks(&self.assignment)
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
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment),
            practice: Scope::new(&d.example),
        })
    }
    pub fn shape(&self) -> Result<()> {
        if !id(&self.source_id) || !bounded(self, 1024 * 1024) {
            return Err("Invalid saved configuration lesson identity/size.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn apply(&mut self, d: &Definition, a: &Action, practice: bool) -> Result<()> {
        d.bound(self)?;
        let mut next = self.clone();
        if practice {
            next.practice.apply(&d.example, a)?;
        } else {
            next.assignment.apply(&d.assignment, a)?;
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<()> {
        d.bound(self)?;
        if practice {
            self.practice = Scope::new(&d.example);
        } else {
            self.assignment = Scope::new(&d.assignment);
        }
        Ok(())
    }
}
