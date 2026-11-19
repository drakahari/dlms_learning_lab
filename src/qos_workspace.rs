//! Pure multi-workload policy grading and isolated drafts. Not a registered lab.
use crate::{qos as core, script_teaching::View};
use serde::{Deserialize, Deserializer, Serialize, de::Visitor};
use std::collections::BTreeSet;
use std::fmt;
pub const MAX_SESSION_BYTES: usize = 16384;
pub const PREFIX: &str = "QOS POLICY\n";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stream {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,100>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,400>")]
    pub description: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workload {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,100>")]
    pub label: String,
    pub fixture: core::Fixture,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Mark {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        flow: String,
        dscp: u8,
    },
    PreserveMark {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        flow: String,
    },
    AvoidPriority {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        flow: String,
    },
    DeliverAll {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        flow: String,
    },
    MaxWait {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        flow: String,
        slots: u16,
    },
    MinDelivered {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        flow: String,
        count: u16,
    },
}
impl Requirement {
    fn key(&self) -> (u8, &str) {
        match self {
            Self::Mark { flow, .. } => (0, flow),
            Self::PreserveMark { flow } => (1, flow),
            Self::AvoidPriority { flow } => (2, flow),
            Self::DeliverAll { flow } => (3, flow),
            Self::MaxWait { flow, .. } => (4, flow),
            Self::MinDelivered { flow, .. } => (5, flow),
        }
    }
    fn check(&self, trace: &core::Trace) -> (bool, String) {
        let flow = self.key().1;
        let packets = trace
            .packets
            .iter()
            .filter(|p| p.flow == flow)
            .collect::<Vec<_>>();
        if packets.is_empty() {
            return (false, format!("{flow}: no supplied packet evidence"));
        }
        let delivered = packets
            .iter()
            .filter(|p| matches!(p.delivery, core::Delivery::Delivered { .. }))
            .count();
        match self {
            Self::Mark { dscp, .. } => {
                let actual = packets
                    .iter()
                    .map(|p| p.outgoing_dscp)
                    .collect::<BTreeSet<_>>();
                let pass = actual.len() == 1 && actual.contains(dscp);
                (
                    pass,
                    format!(
                        "{flow}: required outgoing DSCP {dscp}; actual {actual:?}. Marking changes the label before queue selection."
                    ),
                )
            }
            Self::PreserveMark { .. } => {
                let changed = packets
                    .iter()
                    .filter(|p| p.incoming_dscp != p.outgoing_dscp)
                    .count();
                (
                    changed == 0,
                    format!(
                        "{flow}: keep incoming labels; {changed} of {} were changed.",
                        packets.len()
                    ),
                )
            }
            Self::AvoidPriority { .. } => {
                let promoted = packets
                    .iter()
                    .filter(|p| p.queue == core::Queue::High)
                    .count();
                (
                    promoted == 0,
                    format!(
                        "{flow}: must stay outside the high queue; {promoted} packets were assigned to it. Incoming marks are trusted only when the policy keeps them."
                    ),
                )
            }
            Self::DeliverAll { .. } => (
                delivered == packets.len(),
                format!(
                    "{flow}: deliver all {} packets; actual {delivered}, dropped {}. A full waiting queue drops a new arrival.",
                    packets.len(),
                    packets.len() - delivered
                ),
            ),
            Self::MaxWait { slots, .. } => {
                let max = packets
                    .iter()
                    .filter_map(|p| {
                        if let core::Delivery::Delivered { waiting_slots, .. } = p.delivery {
                            Some(waiting_slots)
                        } else {
                            None
                        }
                    })
                    .max();
                let pass = delivered == packets.len() && max.is_some_and(|n| n <= *slots);
                (
                    pass,
                    format!(
                        "{flow}: deliver every packet with queue waiting at most {slots} slots; actual largest wait {}, delivered {delivered}/{}. A dropped packet does not satisfy a delay requirement.",
                        max.map_or("none".into(), |n| n.to_string()),
                        packets.len()
                    ),
                )
            }
            Self::MinDelivered { count, .. } => (
                delivered >= usize::from(*count),
                format!(
                    "{flow}: deliver at least {count} supplied packets; actual {delivered}/{}.",
                    packets.len()
                ),
            ),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub task: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Stream,4>")]
    pub streams: Vec<Stream>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Workload,4>")]
    pub workloads: Vec<Workload>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,16>")]
    pub requirements: Vec<Requirement>,
    pub starter: core::Policy,
}
pub struct CaseResult {
    pub label: String,
    pub checks: Vec<(bool, String)>,
    pub trace: core::Trace,
}
impl CaseResult {
    pub fn passed(&self) -> bool {
        !self.checks.is_empty() && self.checks.iter().all(|r| r.0)
    }
}
fn short(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max
}
fn identifier(s: &str, max: usize) -> bool {
    short(s, max)
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if !identifier(&self.id, 64)
            || !short(&self.task, 2048)
            || self.streams.is_empty()
            || self.streams.len() > 4
            || self.workloads.len() < 2
            || self.workloads.len() > 4
            || self.requirements.is_empty()
            || self.requirements.len() > 16
        {
            return Err(
                "QoS task needs bounded streams, at least two workloads and outcome requirements"
                    .into(),
            );
        }
        let ids = self
            .streams
            .iter()
            .map(|s| s.id.clone())
            .collect::<BTreeSet<_>>();
        if ids.len() != self.streams.len()
            || self.streams.iter().any(|s| {
                !identifier(&s.id, 32) || !short(&s.label, 100) || !short(&s.description, 400)
            })
        {
            return Err("Invalid or duplicate QoS stream description".into());
        }
        let mut cases = BTreeSet::new();
        let budget = self.workloads[0].fixture.buffer_budget;
        for w in &self.workloads {
            if !identifier(&w.id, 32)
                || !short(&w.label, 100)
                || !cases.insert(w.id.as_str())
                || w.fixture.flows != ids
                || w.fixture.buffer_budget != budget
            {
                return Err(
                    "Workloads need unique IDs, the same supplied flows and the same buffer budget"
                        .into(),
                );
            }
            self.starter.validate(&w.fixture)?;
            if ids
                .iter()
                .any(|flow| !w.fixture.packets.iter().any(|p| &p.flow == flow))
            {
                return Err(
                    "Each workload needs actual packet evidence for every supplied flow".into(),
                );
            }
        }
        let mut keys = BTreeSet::new();
        let mut covered = BTreeSet::new();
        for r in &self.requirements {
            let (kind, flow) = r.key();
            if !ids.contains(flow) || !keys.insert((kind, flow)) {
                return Err("Unknown flow or duplicate QoS outcome".into());
            }
            covered.insert(flow.to_owned());
            match r {
                Requirement::Mark { dscp, .. } if *dscp > 63 => {
                    return Err("DSCP outcome outside 0–63".into());
                }
                Requirement::MaxWait { slots, .. }
                    if *slots > core::MAX_ARRIVAL + core::MAX_PACKETS as u16 =>
                {
                    return Err("Delay outcome exceeds bounded virtual replay".into());
                }
                Requirement::MinDelivered { count, .. }
                    if *count == 0
                        || self.workloads.iter().any(|w| {
                            w.fixture.packets.iter().filter(|p| p.flow == flow).count()
                                < usize::from(*count)
                        }) =>
                {
                    return Err(
                        "Delivery minimum needs positive supplied evidence in every workload"
                            .into(),
                    );
                }
                _ => {}
            }
        }
        if covered != ids {
            return Err("Every editable stream needs an explicit outcome".into());
        }
        Ok(())
    }
    pub fn run(&self, policy: &core::Policy) -> Result<Vec<CaseResult>, String> {
        self.validate()?;
        self.workloads
            .iter()
            .map(|w| {
                let trace = core::replay(&w.fixture, policy)?;
                let checks = self.requirements.iter().map(|r| r.check(&trace)).collect();
                Ok(CaseResult {
                    label: w.label.clone(),
                    checks,
                    trace,
                })
            })
            .collect()
    }
    pub fn draft_bound(&self, p: &core::Policy) -> bool {
        p.marks.len() == self.streams.len()
            && p.marks.iter().all(|(id, m)| {
                self.streams.iter().any(|s| &s.id == id) && !matches!(m,core::Mark::Set(n)if *n>63)
            })
            && p.priority_dscp.len() <= 4
            && p.priority_dscp.iter().all(|n| *n <= 63)
            && p.high_buffer <= core::MAX_BUFFER
            && p.standard_buffer <= core::MAX_BUFFER
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
    pub example_worked: core::Policy,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self
                .initial
                .streams
                .iter()
                .any(|s| self.example.streams.iter().any(|e| e.id == s.id))
        {
            return Err(
                "QoS assignment and practice need distinct origins and flow identities".into(),
            );
        }
        if self
            .initial
            .run(&self.initial.starter)?
            .iter()
            .all(CaseResult::passed)
            || self
                .example
                .run(&self.example.starter)?
                .iter()
                .all(CaseResult::passed)
        {
            return Err("QoS starters must retain meaningful unfinished work".into());
        }
        if !self
            .example
            .run(&self.example_worked)?
            .iter()
            .all(CaseResult::passed)
        {
            return Err("Worked policy does not satisfy every example workload".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        s.validate()?;
        self.validate()?;
        if s.version != 1 || s.origin != self.initial.id {
            return Err("QoS progress belongs to a different source task".into());
        }
        for (scope, task) in [(&s.assignment, &self.initial), (&s.practice, &self.example)] {
            if !task.draft_bound(&scope.draft)
                || scope.diagnostic.as_ref().is_some_and(|d| d.len() > 512)
                || scope.tested.as_ref().is_some_and(|p| {
                    !task
                        .workloads
                        .iter()
                        .all(|w| p.validate(&w.fixture).is_ok())
                })
            {
                return Err("QoS draft or tested policy exceeds its supplied context".into());
            }
        }
        Ok(())
    }
    pub fn solved(&self, s: &Session) -> bool {
        self.bound(s).is_ok() && s.assignment.passed(&self.initial)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        if let Some(e) = &s.assignment.diagnostic {
            return vec![(false, format!("Configuration problem: {e}"))];
        }
        if s.assignment.tested.as_ref() != Some(&s.assignment.draft) {
            return vec![(
                false,
                "Not checked · run the current policy against the supplied workloads".into(),
            )];
        }
        match self.initial.run(&s.assignment.draft) {
            Ok(results) => results
                .into_iter()
                .flat_map(|r| {
                    r.checks
                        .into_iter()
                        .map(move |(ok, text)| (ok, format!("{} · {text}", r.label)))
                })
                .collect(),
            Err(e) => vec![(false, e)],
        }
    }
}
fn diagnostic<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    struct Text;
    impl<'de> Visitor<'de> for Text {
        type Value = Option<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("an optional bounded policy diagnostic")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            crate::cron::text::<D, 512>(d).map(Some)
        }
    }
    d.deserialize_option(Text)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub draft: core::Policy,
    pub tested: Option<core::Policy>,
    #[serde(deserialize_with = "diagnostic")]
    pub diagnostic: Option<String>,
}
impl Scope {
    fn new(t: &Task) -> Self {
        Self {
            draft: t.starter.clone(),
            tested: None,
            diagnostic: None,
        }
    }
    pub fn replace(&mut self, t: &Task, p: core::Policy) -> Result<(), String> {
        t.validate()?;
        if !t.draft_bound(&p) {
            return Err("Draft differs from supplied bounded flows".into());
        }
        if self.draft != p {
            self.draft = p;
            self.tested = None;
            self.diagnostic = None;
        }
        Ok(())
    }
    pub fn run(&mut self, t: &Task) -> Result<Vec<CaseResult>, String> {
        self.tested = None;
        self.diagnostic = None;
        match t.run(&self.draft) {
            Ok(results) => {
                self.tested = Some(self.draft.clone());
                Ok(results)
            }
            Err(e) => {
                let mut end = e.len().min(512);
                while !e.is_char_boundary(end) {
                    end -= 1;
                }
                self.diagnostic = Some(e[..end].into());
                Err(e)
            }
        }
    }
    pub fn passed(&self, t: &Task) -> bool {
        self.diagnostic.is_none()
            && self.tested.as_ref() == Some(&self.draft)
            && t.run(&self.draft)
                .is_ok_and(|r| r.iter().all(CaseResult::passed))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub origin: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    /// Bounds only: unfinished queue settings remain resumable. Actual policy
    /// validity and all outcome grades require the immutable source definition.
    pub fn validate(&self) -> Result<(), String> {
        fn policy(p: &core::Policy) -> bool {
            !p.marks.is_empty()
                && p.marks.len() <= 4
                && p.marks
                    .iter()
                    .all(|(id, m)| identifier(id, 32) && !matches!(m,core::Mark::Set(n)if *n>63))
                && p.priority_dscp.len() <= 4
                && p.priority_dscp.iter().all(|n| *n <= 63)
                && p.high_buffer <= core::MAX_BUFFER
                && p.standard_buffer <= core::MAX_BUFFER
        }
        if self.version != 1
            || !identifier(&self.origin, 64)
            || [&self.assignment, &self.practice].iter().any(|s| {
                !policy(&s.draft)
                    || s.tested.as_ref().is_some_and(|p| !policy(p))
                    || s.diagnostic.as_ref().is_some_and(|e| e.len() > 512)
            })
        {
            Err("QoS progress exceeds its version or bounded draft structure".into())
        } else {
            Ok(())
        }
    }
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        })
    }
    pub fn from_json(text: &str, d: &Definition) -> Result<Self, String> {
        if text.len() > MAX_SESSION_BYTES {
            return Err("QoS progress JSON limit".into());
        }
        let s: Self =
            serde_json::from_str(text).map_err(|_| "Invalid bounded QoS progress".to_owned())?;
        d.bound(&s)?;
        Ok(s)
    }
    /// Data-only authoring/reference adapter; never a host or taught IOS command.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.bound(self)?;
        let raw = input.strip_prefix(PREFIX).ok_or(
            "Use the native traffic policy controls; this adapter is internal reference data",
        )?;
        let policy = core::Policy::from_json(raw, &d.initial.workloads[0].fixture)?;
        self.assignment.replace(&d.initial, policy)?;
        let results = self.assignment.run(&d.initial)?;
        Ok(format!(
            "Simulated policy tested: {}/{} workloads meet every outcome",
            results.iter().filter(|r| r.passed()).count(),
            results.len()
        ))
    }
}

pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (crate::engine::State::Qos { simulation }, crate::engine::Goal::Qos { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("QoS task/state mismatch".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id || definition.initial.task != s.objective {
        return Err("QoS source identity and independent task must match the scenario".into());
    }
    if **simulation != Session::new(definition)? {
        return Err("QoS initial draft differs from its supplied source".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::Qos { simulation } = &a.state else {
        return Err("Saved QoS task belongs to another workspace".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible QoS attempt metadata; original work retained".into());
    }
    d.bound(simulation)
}
