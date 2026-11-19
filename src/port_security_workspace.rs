//! Source-bound secure-port tasks, separate drafts and outcome-based trials.
use crate::{mac_learning::Mac, port_security as core, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, String>;
pub const PREFIX: &str = "SECURE PORT PLAN\n";
pub const MAX_SESSION: usize = 2 * 1024 * 1024;
pub const HELP: &str = "Native port settings and one-frame replay only. Apply settings, Shut down / Bring up, Send frame and Check work operate entirely on virtual Ethernet metadata. No IOS terminal, real packets or host interfaces. Sticky learning, secure aging, automatic recovery and save/reboot are outside this subset.";
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Name(#[serde(deserialize_with = "crate::cron::text::<_,64>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub port: String,
    pub enabled: bool,
    pub maximum: u8,
    pub mode: core::ViolationMode,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Mac,16>")]
    pub configured: Vec<Mac>,
}
impl Settings {
    pub fn from_port(p: &core::Port) -> Self {
        Self {
            port: p.name.clone(),
            enabled: p.enabled,
            maximum: p.maximum,
            mode: p.mode,
            configured: p.configured.clone(),
        }
    }
    pub fn apply(&self, w: &mut core::World) -> Result<()> {
        w.configure(
            &self.port,
            self.enabled,
            self.maximum,
            self.mode,
            self.configured.clone(),
        )
    }
    fn matches(&self, p: &core::Port) -> bool {
        let mut now = Self::from_port(p);
        let mut expected = self.clone();
        now.configured.sort();
        expected.configured.sort();
        now == expected
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Configure { settings: Settings },
    Admin { port: Name, up: bool },
}
impl Action {
    pub fn apply(&self, w: &mut core::World) -> Result<()> {
        match self {
            Self::Configure { settings } => settings.apply(w),
            Self::Admin { port, up } => w.set_admin(&port.0, *up),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Admitted {
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Name,32>")]
        accepted_by: Vec<Name>,
    },
    Violation {
        counted: Option<bool>,
        operational: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub host: Name,
    pub source: Mac,
    pub destination: Mac,
    pub expected: Expected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Frame,8>")]
    pub frames: Vec<Frame>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredPort {
    pub settings: Settings,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,core::ViolationMode,3>")]
    pub modes: Vec<core::ViolationMode>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub id: Name,
    #[serde(deserialize_with = "crate::cron::text::<_,1800>")]
    pub instructions: String,
    pub initial: core::World,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,RequiredPort,8>")]
    pub required: Vec<RequiredPort>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,12>")]
    pub reference: Vec<Action>,
}
fn text(s: &str, n: usize) -> bool {
    !s.trim().is_empty() && s.len() <= n && !s.chars().any(char::is_control)
}
impl Task {
    pub fn compatible(&self, w: &core::World) -> Result<()> {
        w.validate()?;
        if w.baseline() != self.initial.baseline()
            || w.switch() != self.initial.switch()
            || !w
                .ports()
                .iter()
                .map(|p| &p.name)
                .eq(self.initial.ports().iter().map(|p| &p.name))
        {
            return Err(
                "Saved work belongs to different supplied equipment; original retained.".into(),
            );
        }
        Ok(())
    }
    pub fn reference_world(&self) -> Result<core::World> {
        let mut w = self.initial.clone();
        for a in &self.reference {
            a.apply(&mut w)?;
        }
        Ok(w)
    }
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        if !key(&self.id.0)
            || !text(&self.instructions, 1800)
            || self.required.is_empty()
            || self.required.len() > 8
            || !(3..=8).contains(&self.cases.len())
            || self.reference.is_empty()
            || self.reference.len() > 12
        {
            return Err("Secure-port task needs clear supplied work, requirements, three to eight ordered tests and a bounded reachable approach.".into());
        }
        let mut ports = BTreeSet::new();
        for r in &self.required {
            if !ports.insert(&r.settings.port)
                || self.initial.port(&r.settings.port).is_none()
                || r.modes.is_empty()
                || r.modes.len() > 3
                || !r.modes.contains(&r.settings.mode)
            {
                return Err("Required secure ports and permitted modes must be unique, supplied and reachable.".into());
            }
        }
        if ports.len() >= self.initial.ports().len() {
            return Err(
                "Retain at least one neighboring port to protect during this repair.".into(),
            );
        }
        let mut labels = BTreeSet::new();
        let mut allow = false;
        let mut deny = false;
        for c in &self.cases {
            if !text(&c.label, 120)
                || !labels.insert(&c.label)
                || c.frames.is_empty()
                || c.frames.len() > 8
            {
                return Err(
                    "Use unique clear case labels and one to eight frames per case.".into(),
                );
            }
            for f in &c.frames {
                if self.initial.baseline().address(&f.host.0).is_none()
                    || !f.source.unicast()
                    || (!f.destination.unicast() && f.destination != Mac::BROADCAST)
                {
                    return Err(
                        "Use supplied attachments and supported complete frame addresses.".into(),
                    );
                }
                if let Expected::Admitted { accepted_by } = &f.expected {
                    let set: BTreeSet<_> = accepted_by.iter().map(|n| &n.0).collect();
                    if accepted_by.is_empty()
                        || accepted_by.len() > 32
                        || set.len() != accepted_by.len()
                        || set.iter().any(|n| {
                            self.initial.baseline().address(n).is_none() || **n == f.host.0
                        })
                    {
                        return Err("Allowed trials need real supplied destination delivery, not an empty success.".into());
                    }
                    allow = true;
                } else {
                    deny = true;
                }
            }
        }
        let worked = self.reference_world()?;
        if !allow
            || !deny
            || !self.checks(&worked).iter().all(|r| r.0)
            || self.checks(&self.initial).iter().all(|r| r.0)
        {
            return Err("Retain unfinished work and reachable authorized/unwanted-source outcomes; a disabled path is not a security success.".into());
        }
        Ok(())
    }
    pub fn checks(&self, w: &core::World) -> Vec<(bool, String)> {
        if let Err(e) = self.compatible(w) {
            return vec![(false, e)];
        }
        let mut rows = vec![];
        for r in &self.required {
            let p = w.port(&r.settings.port).unwrap();
            let mut required = r.settings.clone();
            required.mode = p.mode;
            rows.push((required.matches(p)&&r.modes.contains(&p.mode)&&p.operational(),format!("{}/{}: {}.{}",w.switch(),p.name,
                if required.matches(p)&&r.modes.contains(&p.mode)&&p.operational(){"required identities, capacity and working port match the task"}else{"check the configured identities, maximum, mode and current port state"},
                if p.error_disabled{" The port is error-disabled; repair the policy, then Shut down and Bring up."}else{""})));
        }
        for p in self
            .initial
            .ports()
            .iter()
            .filter(|p| !self.required.iter().any(|r| r.settings.port == p.name))
        {
            let now = w.port(&p.name).unwrap();
            rows.push((
                Settings::from_port(p).matches(now)
                    && now.admin_up == p.admin_up
                    && now.error_disabled == p.error_disabled,
                format!(
                    "{}/{}: retain the neighboring port's settings and operational state.",
                    w.switch(),
                    p.name
                ),
            ));
        }
        for c in &self.cases {
            let mut trial = w.clone();
            let mut failures = vec![];
            for (i, f) in c.frames.iter().enumerate() {
                let before = trial
                    .ports()
                    .iter()
                    .map(|p| (&p.name, p.violations))
                    .collect::<std::collections::BTreeMap<_, _>>();
                // Own these values before mutating the trial, not borrowed proof.
                let before: std::collections::BTreeMap<_, _> =
                    before.into_iter().map(|(n, c)| (n.clone(), c)).collect();
                let result = trial.send(&f.host.0, f.source, f.destination);
                let passed = match (&f.expected, &result) {
                    (
                        Expected::Admitted { accepted_by },
                        Ok(core::Outcome::Admitted { forwarding, .. }),
                    ) => {
                        forwarding.accepted_by == accepted_by.iter().map(|n| n.0.clone()).collect()
                    }
                    (
                        Expected::Violation {
                            counted,
                            operational,
                        },
                        Ok(core::Outcome::Violation {
                            port,
                            counter,
                            mode,
                            ..
                        }),
                    ) => {
                        let prev = before[port];
                        let p = trial.port(port).unwrap();
                        p.operational() == *operational
                            && counted.is_none_or(|yes| {
                                yes == (*mode != core::ViolationMode::Protect)
                                    && *counter
                                        == if yes {
                                            prev.saturating_add(1).min(core::MAX_COUNTER)
                                        } else {
                                            prev
                                        }
                            })
                    }
                    _ => false,
                };
                if !passed {
                    let expected = match &f.expected {
                        Expected::Admitted { accepted_by } => format!(
                            "delivery to {}",
                            accepted_by
                                .iter()
                                .map(|n| n.0.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        Expected::Violation {
                            counted,
                            operational,
                        } => format!(
                            "a security rejection{}, with the port {}",
                            match counted {
                                Some(true) => " counted",
                                Some(false) => " not counted",
                                None => "",
                            },
                            if *operational {
                                "still working"
                            } else {
                                "error-disabled"
                            }
                        ),
                    };
                    let concise = match (&f.expected, &result) {
                        (Expected::Violation{counted:Some(true),..},Ok(core::Outcome::Violation{mode:core::ViolationMode::Protect,..})) =>
                            Some("expected a counted rejection; actual Protect rejected it without counting. Review the violation mode.".into()),
                        (Expected::Violation{operational:true,..},Ok(core::Outcome::Violation{mode:core::ViolationMode::Shutdown,..})) =>
                            Some("expected rejection while staying online; actual Shutdown error-disabled the port. Approved traffic must continue.".into()),
                        (_,Ok(core::Outcome::Unavailable{..})) =>
                            Some("actual result: unavailable attachment. No new security decision occurred; check the port state and manual recovery.".into()),
                        (Expected::Violation{..},Ok(core::Outcome::Admitted{..})) =>
                            Some("expected rejection; actual source was admitted. Check reserved identities and any free learning slots.".into()),
                        (Expected::Admitted{..},Ok(core::Outcome::Violation{cause,..})) =>
                            Some(format!("expected approved delivery; actual source was rejected: {}.",match cause{core::Cause::Capacity=>"no secure slot is available for it".into(),core::Cause::SecuredElsewhere{port}=>format!("this address is secured on {port} in the same VLAN")})),
                        _ => None,
                    };
                    let actual = match result {
                        Ok(o) => describe(&o),
                        Err(e) => format!("input/model error: {e}"),
                    };
                    let detail =
                        concise.unwrap_or_else(|| format!("expected {expected}; actual {actual}"));
                    failures.push(format!("Frame {}: {detail}", i + 1));
                }
            }
            rows.push((
                failures.is_empty(),
                format!(
                    "{}: {}",
                    c.label,
                    if failures.is_empty() {
                        "every frame matches".into()
                    } else {
                        failures.join(". ")
                    }
                ),
            ));
        }
        rows
    }
}
pub fn describe(o: &core::Outcome) -> String {
    match o {
        core::Outcome::Unavailable { reason } => format!("unavailable attachment — {reason}"),
        core::Outcome::Violation {
            port,
            source,
            cause,
            mode,
            counter,
            ..
        } => format!(
            "security rejection on {port} for {}: {}; {mode:?}, counter {counter}",
            source.text(),
            match cause {
                core::Cause::Capacity => "the secure-address capacity is full".into(),
                core::Cause::SecuredElsewhere { port } =>
                    format!("this address is secured on {port} in the same VLAN"),
            }
        ),
        core::Outcome::Admitted {
            learned,
            forwarding,
        } => format!(
            "admitted{}; received by {}",
            if learned.is_some() {
                " and learned securely"
            } else {
                ""
            },
            if forwarding.accepted_by.is_empty() {
                "no workstation".into()
            } else {
                let names = forwarding
                    .accepted_by
                    .iter()
                    .take(4)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                let extra = forwarding.accepted_by.len().saturating_sub(4);
                if extra > 0 {
                    format!("{names} (+{extra} others)")
                } else {
                    names
                }
            }
        ),
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
        if self.initial.id == self.example.id
            || self.initial.initial.switch() == self.example.initial.switch()
            || self
                .initial
                .initial
                .baseline()
                .fabric()
                .hosts
                .keys()
                .any(|h| self.example.initial.baseline().address(h).is_some())
        {
            return Err("Worked practice needs separate supplied equipment and identities.".into());
        }
        Ok(())
    }
    pub fn from_json(raw: &str) -> Result<Self> {
        if raw.len() > 256 * 1024 {
            return Err("Secure-port definition exceeds 256 KiB.".into());
        }
        let d: Self = serde_json::from_str(raw).map_err(|_| "Invalid secure-port definition.")?;
        d.validate()?;
        Ok(d)
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        self.validate()?;
        s.validate()?;
        if s.origin != self.initial.id {
            return Err("Saved secure-port work belongs to another task.".into());
        }
        self.initial.compatible(&s.assignment.world)?;
        self.example.compatible(&s.practice.world)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        if !s.assignment.checked || !s.assignment.applied() {
            return vec![(
                false,
                "Not checked · Apply current settings, then Check work.".into(),
            )];
        }
        self.initial.checks(&s.assignment.world)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub port: Name,
    pub enabled: bool,
    pub mode: core::ViolationMode,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub maximum: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub configured: String,
}
impl Draft {
    pub fn new(p: &core::Port) -> Self {
        Self {
            port: Name(p.name.clone()),
            enabled: p.enabled,
            mode: p.mode,
            maximum: p.maximum.to_string(),
            configured: p
                .configured
                .iter()
                .map(|m| m.text())
                .collect::<Vec<_>>()
                .join(", "),
        }
    }
    pub fn settings(&self) -> Result<Settings> {
        if self.maximum.len() > 64 || self.configured.len() > 1024 {
            return Err("Port input exceeds its safe size.".into());
        }
        let maximum = self
            .maximum
            .trim()
            .parse::<u8>()
            .map_err(|_| "Maximum: enter a whole number from 1 to 16.")?;
        let configured = if self.configured.trim().is_empty() {
            vec![]
        } else {
            self.configured
                .split(',')
                .map(|s| Mac::parse(s.trim()))
                .collect::<Result<Vec<_>>>()?
        };
        if configured.len() > 16 {
            return Err(
                "Configured addresses: at most 16 complete MAC addresses separated by commas."
                    .into(),
            );
        }
        Ok(Settings {
            port: self.port.0.clone(),
            enabled: self.enabled,
            maximum,
            mode: self.mode,
            configured,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub text: String,
    pub error: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub world: core::World,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Draft,8>")]
    pub drafts: Vec<Draft>,
    pub host: Name,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub destination: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Line,8>")]
    pub output: Vec<Line>,
    pub checked: bool,
}
impl Scope {
    pub fn new(t: &Task) -> Self {
        let f = &t.cases[0].frames[0];
        Self {
            world: t.initial.clone(),
            drafts: t.initial.ports().iter().map(Draft::new).collect(),
            host: f.host.clone(),
            source: f.source.text(),
            destination: f.destination.text(),
            output: vec![],
            checked: false,
        }
    }
    pub fn validate(&self) -> Result<()> {
        self.world.validate()?;
        if !self
            .drafts
            .iter()
            .map(|d| &d.port.0)
            .eq(self.world.ports().iter().map(|p| &p.name))
            || self.drafts.iter().any(|d| {
                d.maximum.len() > 64
                    || d.configured.len() > 1024
                    || d.maximum
                        .chars()
                        .chain(d.configured.chars())
                        .any(char::is_control)
            })
            || self.world.baseline().address(&self.host.0).is_none()
            || self.source.len() > 64
            || self.destination.len() > 64
            || self
                .source
                .chars()
                .chain(self.destination.chars())
                .any(char::is_control)
            || self.output.len() > 8
            || self.output.iter().any(|l| l.text.len() > 2048)
        {
            return Err(
                "Secure-port drafts or replay exceed their supplied identities and bounds.".into(),
            );
        }
        Ok(())
    }
    pub fn applied(&self) -> bool {
        self.drafts.iter().all(|d| {
            d.settings()
                .is_ok_and(|s| self.world.port(&s.port).is_some_and(|p| s.matches(p)))
        })
    }
    pub fn apply(&mut self) -> Result<()> {
        self.validate()?;
        self.checked = false;
        let mut next = self.world.clone();
        for d in &self.drafts {
            d.settings()?.apply(&mut next)?;
        }
        self.world = next;
        Ok(())
    }
    pub fn admin(&mut self, port: &str, up: bool) -> Result<()> {
        self.validate()?;
        self.checked = false;
        self.world.set_admin(port, up)
    }
    pub fn send(&mut self) -> Result<()> {
        self.validate()?;
        self.checked = false;
        let parse = |label: &str, value: &str| {
            if value.trim().is_empty() {
                Err(format!("{label} is blank; enter six hexadecimal pairs."))
            } else {
                Mac::parse(value.trim()).map_err(|e| format!("{label}: {e}"))
            }
        };
        let result = parse("Source MAC", &self.source).and_then(|m| {
            parse("Destination MAC", &self.destination)
                .and_then(|to| self.world.send(&self.host.0, m, to))
        });
        let (text, error) = match result {
            Ok(ref o) => (describe(o), false),
            Err(ref e) => (format!("Input needs attention: {e}"), true),
        };
        let text = format!(
            "{} · {} → {}: {text}",
            self.host.0,
            self.source.trim(),
            self.destination.trim()
        );
        self.output.push(Line { text, error });
        if self.output.len() > 8 {
            self.output.remove(0);
        }
        result.map(|_| ())
    }
    pub fn check(&mut self, t: &Task) -> Result<Vec<(bool, String)>> {
        self.validate()?;
        t.compatible(&self.world)?;
        self.checked = self.applied();
        if !self.checked {
            return Err(
                "Apply the current port settings before checking; previous applied state remains."
                    .into(),
            );
        }
        Ok(t.checks(&self.world))
    }
    pub fn fill(&mut self, t: &Task) -> Result<()> {
        t.compatible(&self.world)?;
        self.world = t.reference_world()?;
        self.drafts = self.world.ports().iter().map(Draft::new).collect();
        self.checked = false;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u8,
    pub origin: Name,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        })
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || !key(&self.origin.0) {
            return Err("Unknown secure-port progress version or identity.".into());
        }
        self.assignment.validate()?;
        self.practice.validate()
    }
    pub fn from_json(raw: &str, d: &Definition) -> Result<Self> {
        if raw.len() > MAX_SESSION {
            return Err("Secure-port progress exceeds 2 MiB.".into());
        }
        let s: Self = serde_json::from_str(raw).map_err(|_| "Invalid secure-port progress.")?;
        d.bound(&s)?;
        Ok(s)
    }
}

impl Session {
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String> {
        d.bound(self)?;
        let raw = input.strip_prefix(PREFIX).ok_or(
            "Use native secure-port controls; this internal data plan is not an IOS command.",
        )?;
        if raw.len() > 16 * 1024 {
            return Err("Secure-port action plan exceeds 16 KiB.".into());
        }
        #[derive(Deserialize)]
        struct Plan(
            #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,12>")]
            Vec<Action>,
        );
        let Plan(actions) =
            serde_json::from_str(raw).map_err(|_| "Invalid secure-port action plan.")?;
        if actions.is_empty() {
            return Err("Secure-port action plan is empty.".into());
        }
        let mut next = self.assignment.clone();
        for a in actions {
            a.apply(&mut next.world)?;
        }
        next.drafts = next.world.ports().iter().map(Draft::new).collect();
        let rows = next.check(&d.initial)?;
        self.assignment = next;
        Ok(format!(
            "Secure-port work checked: {}/{} requirements match",
            rows.iter().filter(|r| r.0).count(),
            rows.len()
        ))
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::PortSecurity { simulation },
        crate::engine::Goal::PortSecurity { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Secure-port source/state mismatch.".into());
    };
    definition.validate()?;
    if definition.initial.id.0 != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err("Secure-port source identity, task and supplied equipment must match.".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::PortSecurity { simulation } = &a.state else {
        return Err("Secure-port work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible secure-port attempt metadata; original retained.".into());
    }
    d.bound(simulation)
}
