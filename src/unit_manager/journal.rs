//! Retained virtual job events. Historical display only; never success authority.
use super::{
    ErrorKind, Inventory, JobKind, JobResult, MAX_ERROR_BYTES, MAX_UNITS, Report, Result, Runtime,
    State, error, invalid, text, text128, unit,
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, SeqAccess, Visitor},
};
use std::{collections::BTreeMap, fmt};
pub const MAX_EVENTS: usize = 256;
fn detail<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    text::<D, MAX_ERROR_BYTES>(d)
}
fn events<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<Event>, D::Error> {
    struct Events;
    impl<'de> Visitor<'de> for Events {
        type Value = Vec<Event>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most256 typed virtual journal events")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while out.len() < MAX_EVENTS {
                match a.next_element()? {
                    Some(e) => out.push(e),
                    None => return Ok(out),
                }
            }
            if a.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(A::Error::custom("Virtual journal event count limit"));
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Events)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum EventKind {
    SimpleDispatched,
    TargetActivated,
    OneshotCompleted {
        remaining: bool,
    },
    ProgramInitialized,
    Stopped,
    StartFailed {
        fault: ErrorKind,
        #[serde(deserialize_with = "detail")]
        detail: String,
    },
    InitializationFailed {
        fault: ErrorKind,
        #[serde(deserialize_with = "detail")]
        detail: String,
    },
    DependencyFailed {
        #[serde(deserialize_with = "text128")]
        dependency: String,
    },
}
impl EventKind {
    fn bounded(&self) -> bool {
        match self {
            Self::StartFailed { detail, .. } | Self::InitializationFailed { detail, .. } => {
                detail.len() <= MAX_ERROR_BYTES
                    && !detail.trim().is_empty()
                    && !detail.chars().any(char::is_control)
            }
            Self::DependencyFailed { dependency } => unit(dependency),
            _ => true,
        }
    }
    /// Original generated simulator messages, without fabricated process/time fields.
    fn message(&self, unit: &str) -> String {
        match self {
            Self::SimpleDispatched => {
                format!("Start job completed for {unit}; virtual program initialization pending.")
            }
            Self::TargetActivated => format!("Activated virtual target {unit}."),
            Self::OneshotCompleted { remaining } => format!(
                "Completed virtual oneshot {unit}; {}.",
                if *remaining {
                    "remains active (exited)"
                } else {
                    "inactive after successful exit"
                }
            ),
            Self::ProgramInitialized => {
                format!("Virtual program for {unit} initialized successfully.")
            }
            Self::Stopped => format!("Stopped virtual unit {unit}."),
            Self::StartFailed { detail, .. } => {
                format!("Virtual start job for {unit} failed: {detail}")
            }
            Self::InitializationFailed { detail, .. } => {
                format!("Virtual program for {unit} could not initialize: {detail}")
            }
            Self::DependencyFailed { dependency } => format!(
                "Virtual start job for {unit} blocked by failed ordered requirement {dependency}."
            ),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    sequence: u64,
    #[serde(deserialize_with = "text128")]
    unit: String,
    event: EventKind,
}
impl Event {
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn unit(&self) -> &str {
        &self.unit
    }
    pub fn kind(&self) -> &EventKind {
        &self.event
    }
    pub fn message(&self) -> String {
        if !unit(&self.unit) || !self.event.bounded() {
            return "Invalid virtual journal entry".into();
        }
        self.event.message(&self.unit)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Journal {
    version: u8,
    last: u64,
    dropped: u64,
    events: Vec<Event>,
}
impl<'de> Deserialize<'de> for Journal {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Data {
            version: u8,
            last: u64,
            dropped: u64,
            #[serde(deserialize_with = "events")]
            events: Vec<Event>,
        }
        let data = Data::deserialize(d)?;
        let journal = Self {
            version: data.version,
            last: data.last,
            dropped: data.dropped,
            events: data.events,
        };
        if !journal.bounded() {
            return Err(D::Error::custom(
                "Invalid virtual journal shape or sequence",
            ));
        }
        Ok(journal)
    }
}
impl Journal {
    fn empty() -> Self {
        Self {
            version: 1,
            last: 0,
            dropped: 0,
            events: vec![],
        }
    }
    pub fn events(&self) -> &[Event] {
        &self.events
    }
    pub fn dropped(&self) -> u64 {
        self.dropped
    }
    fn bounded(&self) -> bool {
        self.version == 1
            && self.events.len() <= MAX_EVENTS
            && self.dropped.checked_add(self.events.len() as u64) == Some(self.last)
            && (self.events.is_empty() && self.last == 0 || !self.events.is_empty())
            && self.events.iter().enumerate().all(|(i, e)| {
                self.dropped.checked_add(i as u64 + 1) == Some(e.sequence)
                    && unit(&e.unit)
                    && e.event.bounded()
            })
    }
    pub(super) fn validate(&self, inventory: &Inventory) -> Result<()> {
        if !self.bounded()
            || self.events.iter().any(|e| {
                !inventory.units.contains(&e.unit)
                    || match &e.event {
                        EventKind::DependencyFailed { dependency } => {
                            dependency == &e.unit || !inventory.units.contains(dependency)
                        }
                        EventKind::TargetActivated => !e.unit.ends_with(".target"),
                        EventKind::SimpleDispatched
                        | EventKind::OneshotCompleted { .. }
                        | EventKind::ProgramInitialized
                        | EventKind::InitializationFailed { .. } => !e.unit.ends_with(".service"),
                        _ => false,
                    }
            })
        {
            return Err(invalid("Invalid bounded virtual journal context"));
        }
        Ok(())
    }
    fn append(&mut self, payloads: Vec<(String, EventKind)>) -> Result<()> {
        if !self.bounded()
            || payloads.len() > MAX_UNITS
            || payloads.iter().any(|(u, e)| !unit(u) || !e.bounded())
        {
            return Err(invalid("Invalid virtual journal append"));
        }
        let last = self
            .last
            .checked_add(payloads.len() as u64)
            .ok_or_else(|| {
                error(
                    ErrorKind::Limit,
                    "Virtual journal sequence limit; reset selected attempt to begin a new window",
                )
            })?;
        let evicted = (self.events.len() + payloads.len()).saturating_sub(MAX_EVENTS);
        let dropped = self
            .dropped
            .checked_add(evicted as u64)
            .ok_or_else(|| error(ErrorKind::Limit, "Virtual journal retention counter limit"))?;
        for (unit, event) in payloads {
            self.last += 1;
            self.events.push(Event {
                sequence: self.last,
                unit,
                event,
            });
        }
        if evicted > 0 {
            self.events.drain(..evicted);
        }
        self.last = last;
        self.dropped = dropped;
        Ok(())
    }
}
/// Record only actual job results in a bounded candidate. A caller publishes
/// that candidate only after this operation and full State validation succeed.
pub(super) fn record(
    before: &BTreeMap<String, Runtime>,
    after: &mut State,
    report: &Report,
) -> Result<()> {
    if report.jobs.len() > MAX_UNITS {
        return Err(invalid("Virtual journal job report bound"));
    }
    let mut payloads = vec![];
    for job in &report.jobs {
        if !after.inventory.units.contains(&job.unit) {
            return Err(invalid("Foreign unit in virtual journal job"));
        }
        let event = match (&job.kind, &job.result) {
            (_, JobResult::AlreadyActive) => None,
            (JobKind::Start, JobResult::Done) => Some(match after.runtime.get(&job.unit) {
                Some(Runtime::Dispatched { .. }) => EventKind::SimpleDispatched,
                Some(Runtime::TargetActive {}) => EventKind::TargetActivated,
                Some(Runtime::Exited { .. }) => EventKind::OneshotCompleted { remaining: true },
                Some(Runtime::Inactive {}) => EventKind::OneshotCompleted { remaining: false },
                _ => return Err(invalid("Start result and virtual journal phase differ")),
            }),
            (JobKind::Initialize, JobResult::Done) => {
                if !matches!(after.runtime.get(&job.unit), Some(Runtime::Running { .. })) {
                    return Err(invalid(
                        "Initialization result and virtual journal phase differ",
                    ));
                }
                Some(EventKind::ProgramInitialized)
            }
            (JobKind::Stop, JobResult::Done) => {
                if !matches!(after.runtime.get(&job.unit), Some(Runtime::Inactive {})) {
                    return Err(invalid("Stop result and virtual journal phase differ"));
                }
                before
                    .get(&job.unit)
                    .filter(|r| !matches!(r, Runtime::Inactive {}))
                    .map(|_| EventKind::Stopped)
            }
            (JobKind::Start, JobResult::Failed { kind, message })
            | (JobKind::Initialize, JobResult::Failed { kind, message }) => {
                if message.len() > MAX_ERROR_BYTES || message.chars().any(char::is_control) {
                    return Err(invalid("Virtual journal diagnostic bound"));
                }
                Some(if matches!(job.kind, JobKind::Start) {
                    EventKind::StartFailed {
                        fault: *kind,
                        detail: message.clone(),
                    }
                } else {
                    EventKind::InitializationFailed {
                        fault: *kind,
                        detail: message.clone(),
                    }
                })
            }
            (JobKind::Start, JobResult::DependencyFailed { unit }) => {
                if unit == &job.unit || !after.inventory.units.contains(unit) {
                    return Err(invalid("Foreign dependency in virtual journal job"));
                }
                Some(EventKind::DependencyFailed {
                    dependency: unit.clone(),
                })
            }
            _ => return Err(invalid("Unsupported virtual journal job result")),
        };
        if let Some(event) = event {
            payloads.push((job.unit.clone(), event));
        }
    }
    if payloads.is_empty() {
        return Ok(());
    }
    let journal = after.journal.get_or_insert_with(Journal::empty);
    journal.append(payloads)
}
