//! Source-bound print tasks and finite behavioral cases. No authored content or
//! learner-host execution is implied by this capability module.
use crate::print_queues::{Facts, JobState, QueueName, World, command};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_HISTORY: usize = 96;
pub mod bindings;
pub use bindings::{context, relevant, source};
pub const HELP: &str = crate::print_queues::command::HELP;
pub mod session;
pub use session::{Definition, Session};
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Command(#[serde(deserialize_with = "crate::cron::text::<_,1024>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Path(#[serde(deserialize_with = "crate::cron::text::<_,128>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrinterRequirement {
    pub printer: QueueName,
    pub accepting: bool,
    pub enabled: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExistingJob {
    pub id: u32,
    pub state: JobState,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Delivery {
    pub document: Path,
    pub printer: QueueName,
    pub copies: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DefaultRequirement {
    Preserve {},
    Use { printer: QueueName },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Required {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,PrinterRequirement,8>")]
    pub printers: Vec<PrinterRequirement>,
    pub default: DefaultRequirement,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,ExistingJob,64>")]
    pub existing_jobs: Vec<ExistingJob>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Delivery,8>")]
    pub deliveries: Vec<Delivery>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expected {
    Printed,
    Pending,
    Held,
    Rejected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub label: String,
    pub destination: Option<QueueName>,
    pub document: Path,
    pub copies: u16,
    pub held: bool,
    pub expected: Expected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub facts: Facts,
    pub initial: World,
    pub required: Required,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Command,32>")]
    pub reference: Vec<Command>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub label: String,
    pub correct: bool,
    pub feedback: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
}
fn row(label: impl Into<String>, correct: bool, yes: &str, no: impl Into<String>) -> Check {
    Check {
        label: label.into(),
        correct,
        feedback: if correct { yes.into() } else { no.into() },
        expected: None,
        actual: None,
    }
}
fn printer_text(accepting: bool, enabled: bool) -> String {
    format!(
        "{} new requests; {}",
        if accepting { "Accepting" } else { "Rejecting" },
        if enabled {
            "Enabled for printing"
        } else {
            "Stopped"
        }
    )
}
impl Task {
    fn shape(&self) -> Result<()> {
        if self.id.is_empty()
            || self.id.len() > 64
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
        {
            return Err("Use a short stable print-task identifier.".into());
        }
        self.initial
            .validate(&self.facts)
            .map_err(|e| e.to_string())?;
        if self.required.printers.len() > 8
            || self.required.existing_jobs.len() > 64
            || self.required.deliveries.len() > 8
            || !(3..=8).contains(&self.cases.len())
            || self.reference.is_empty()
            || self.reference.len() > 32
        {
            return Err("Print tasks need bounded requirements,3–8 distinct behavior cases and a reachable method.".into());
        }
        let mut names = BTreeSet::new();
        for p in &self.required.printers {
            self.initial
                .printer(&p.printer.0)
                .map_err(|e| e.to_string())?;
            if !names.insert(p.printer.0.to_ascii_lowercase()) {
                return Err("Duplicate required printer.".into());
            }
        }
        if let DefaultRequirement::Use { printer } = &self.required.default {
            self.initial
                .printer(&printer.0)
                .map_err(|e| e.to_string())?;
        }
        let mut ids = BTreeSet::new();
        for j in &self.required.existing_jobs {
            if !ids.insert(j.id) || !self.initial.jobs.iter().any(|old| old.id == j.id) {
                return Err(
                    "Requirements must identify distinct supplied existing requests.".into(),
                );
            }
        }
        let mut pairs = BTreeSet::new();
        for d in &self.required.deliveries {
            self.facts
                .document(&d.document.0)
                .map_err(|e| e.to_string())?;
            self.initial
                .printer(&d.printer.0)
                .map_err(|e| e.to_string())?;
            if !(1..=40).contains(&d.copies)
                || !pairs.insert((d.document.0.clone(), d.printer.0.to_ascii_lowercase()))
            {
                return Err(
                    "Use distinct document/destination delivery requirements of1–40 copies.".into(),
                );
            }
        }
        let mut labels = BTreeSet::new();
        let mut inputs = BTreeSet::new();
        for c in &self.cases {
            if c.label.is_empty()
                || c.label.len() > 64
                || c.label.chars().any(char::is_control)
                || !labels.insert(&c.label)
            {
                return Err("Behavior cases need distinct readable labels.".into());
            }
            if let Some(p) = &c.destination {
                self.initial.printer(&p.0).map_err(|e| e.to_string())?;
            }
            self.facts
                .document(&c.document.0)
                .map_err(|e| e.to_string())?;
            if !(1..=20).contains(&c.copies)
                || !inputs.insert((
                    c.destination.as_ref().map(|p| p.0.to_ascii_lowercase()),
                    c.document.0.clone(),
                    c.copies,
                    c.held,
                ))
            {
                return Err(
                    "Use distinct supported print requests as deterministic test inputs.".into(),
                );
            }
        }
        if self
            .reference
            .iter()
            .any(|c| c.0.is_empty() || c.0.len() > 1024)
        {
            return Err("Reference commands must fit the literal command bounds.".into());
        }
        Ok(())
    }
    pub fn replay(&self, history: &[Command]) -> Result<World> {
        self.shape()?;
        if history.len() > MAX_HISTORY {
            return Err("Print history exceeds96commands; original retained.".into());
        }
        let mut world = self.initial.clone();
        for c in history {
            command::run(&mut world, &self.facts, &c.0)
                .map_err(|e| format!("Saved print command cannot be replayed: {e}"))?;
        }
        Ok(world)
    }
    pub fn grade(&self, world: &World) -> Vec<Check> {
        if let Err(error) = self
            .shape()
            .and_then(|()| world.validate(&self.facts).map_err(|e| e.to_string()))
        {
            return vec![row("Supplied print workspace", false, "", error)];
        }
        let mut checks = vec![];
        for original in &self.initial.printers {
            let current = world.printer(&original.name.0).expect("validated identity");
            if let Some(wanted) = self
                .required
                .printers
                .iter()
                .find(|p| p.printer.0.eq_ignore_ascii_case(&original.name.0))
            {
                let correct =
                    current.accepting == wanted.accepting && current.enabled == wanted.enabled;
                checks.push(row(format!("Printer · {}",original.name.0),correct,"The printer has the required acceptance and processing states.",format!("Current: {}. Required: {}. Accepting new requests does not start processing; enabling a printer does not release individually held jobs.",printer_text(current.accepting,current.enabled),printer_text(wanted.accepting,wanted.enabled))));
            } else {
                checks.push(row(format!("Preserved printer · {}",original.name.0),current==original,"The unrelated printer settings are unchanged.","This printer's settings changed. Preserve its original acceptance, processing and reason fields."));
            }
        }
        let expected_default = match &self.required.default {
            DefaultRequirement::Preserve {} => self.initial.local_default.as_ref(),
            DefaultRequirement::Use { printer } => Some(printer),
        };
        let default_correct = match (&world.local_default, expected_default) {
            (None, None) => true,
            (Some(a), Some(b)) => a.0.eq_ignore_ascii_case(&b.0),
            _ => false,
        };
        checks.push(row("Default destination",default_correct,"The effective local default meets the task.","Check the local default. Explicitly choosing a destination for one request does not change where later default requests go."));
        for old in &self.initial.jobs {
            let required = self
                .required
                .existing_jobs
                .iter()
                .find(|r| r.id == old.id)
                .map_or(old.state, |r| r.state);
            let correct = world.jobs.iter().find(|j| j.id == old.id).is_some_and(|j| {
                j.printer == old.printer
                    && j.document == old.document
                    && j.copies == old.copies
                    && j.state == required
            });
            checks.push(row(format!("Existing request · {}-{}",old.printer.0,old.id),correct,"The existing request has the required state and original document/copies.",format!("This existing request must be {required:?} while retaining its destination, document and copy count. A new replacement request does not preserve its identity.")));
        }
        for wanted in &self.required.deliveries {
            let before = self
                .initial
                .completed_copies(&wanted.document.0, &wanted.printer.0);
            let actual = world
                .completed_copies(&wanted.document.0, &wanted.printer.0)
                .saturating_sub(before);
            checks.push(row(format!("Delivered copies · {} → {}",wanted.document.0,wanted.printer.0),actual==u32::from(wanted.copies),"The actual modeled requests delivered the requested number of copies.",format!("Newly completed copies: {actual}; required: {}. Inspect request status and destination. A held or pending request has not printed; too many or wrong-destination copies are not success.",wanted.copies)));
        }
        let source_ids = self
            .initial
            .jobs
            .iter()
            .map(|j| j.id)
            .collect::<BTreeSet<_>>();
        let unexpected = world
            .jobs
            .iter()
            .filter(|j| !source_ids.contains(&j.id))
            .any(|j| match j.state {
                JobState::Canceled => false,
                JobState::Completed => !self.required.deliveries.iter().any(|d| {
                    d.document.0 == j.document && d.printer.0.eq_ignore_ascii_case(&j.printer.0)
                }),
                JobState::Pending | JobState::Held => true,
            });
        checks.push(row("Additional requests",!unexpected,"No unintended completed or unfinished new request remains.","An additional request is still held/pending or printed an unintended document/destination. Withdraw an unfinished mistake; an already printed mistake requires resetting this attempt, not deleting history."));
        for c in &self.cases {
            let mut sample = world.clone();
            let destination = c.destination.as_ref().map(|p| p.0.as_str());
            let rejecting = destination
                .or_else(|| sample.local_default.as_ref().map(|p| p.0.as_str()))
                .and_then(|p| sample.printer(p).ok())
                .is_some_and(|p| !p.accepting);
            let result = sample.submit(&self.facts, destination, &c.document.0, c.copies, c.held);
            let (correct, actual) = match result {
                Ok(id) => {
                    let job = sample.request(&id.to_string()).unwrap();
                    let wanted = match c.expected {
                        Expected::Printed => Some(JobState::Completed),
                        Expected::Pending => Some(JobState::Pending),
                        Expected::Held => Some(JobState::Held),
                        Expected::Rejected => None,
                    };
                    (
                        wanted == Some(job.state),
                        format!(
                            "{}-{id}: {:?}; {} copies of {}",
                            job.printer.0, job.state, job.copies, job.document
                        ),
                    )
                }
                Err(e) => (
                    c.expected == Expected::Rejected
                        && rejecting
                        && world.jobs.len() < crate::print_queues::MAX_JOBS
                        && world.next_id < 1_000_000,
                    format!("Request refused: {e}"),
                ),
            };
            let mut check = row(
                format!("Fresh request case · {}", c.label),
                correct,
                "This separate sample request has the intended outcome.",
                "This fresh sample did not behave as required. Compare its destination/default, acceptance, printer processing and individual hold; the case does not change your assignment.",
            );
            check.expected = Some(format!(
                "{} · {} copies of {}",
                match c.expected {
                    Expected::Printed => "Completed in the simulator",
                    Expected::Pending => "Queued and waiting for the printer",
                    Expected::Held => "Individually held",
                    Expected::Rejected => "Rejected by the queue",
                },
                c.copies,
                c.document.0
            ));
            check.actual = Some(actual);
            checks.push(check);
        }
        checks
    }
    pub fn validate(&self) -> Result<()> {
        self.shape()?;
        if self.grade(&self.initial).iter().all(|c| c.correct) {
            return Err(
                "The independent print task must require real changes, not begin solved.".into(),
            );
        }
        let solved = self.replay(&self.reference)?;
        if self.grade(&solved).iter().any(|c| !c.correct) {
            return Err(
                "The taught print method does not reach all required outcomes and fresh cases."
                    .into(),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "print_queue_workspace/tests.rs"]
mod tests;
