//! Finite in-memory print-queue foundation; not yet authored curriculum.
//! No daemon, device, host file, subprocess or network access.
use crate::boot_entry::{Error, ErrorKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub type Result<T> = std::result::Result<T, Error>;
pub const MAX_JOBS: usize = 64;
fn invalid(message: impl Into<String>) -> Error {
    Error {
        kind: ErrorKind::Invalid,
        message: message.into(),
    }
}
fn bounded(s: &str, n: usize) -> bool {
    !s.is_empty() && s.len() <= n && !s.chars().any(char::is_control)
}
pub fn queue_name(s: &str) -> bool {
    bounded(s, 40)
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
fn file_name(s: &str) -> bool {
    crate::boot_entry::path(s)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub title: String,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub text: String,
    /// Supplied modeled page count; never presented as a real text filter calculation.
    pub pages: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QueueName(#[serde(deserialize_with = "crate::cron::text::<_,40>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,QueueName,8>")]
    pub printers: Vec<QueueName>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Document,8>")]
    pub documents: Vec<Document>,
    #[serde(deserialize_with = "crate::cron::text::<_,40>")]
    pub operator: String,
}
impl Facts {
    pub fn validate(&self) -> Result<()> {
        if self.printers.is_empty()
            || self.printers.len() > 8
            || self.documents.is_empty()
            || self.documents.len() > 8
            || !bounded(&self.operator, 40)
        {
            return Err(invalid(
                "Supply 1–8 virtual printers/documents and a bounded operator identity.",
            ));
        }
        let mut printers = BTreeSet::new();
        for name in &self.printers {
            if !queue_name(&name.0) || !printers.insert(name.0.to_ascii_lowercase()) {
                return Err(invalid(
                    "Use unique ordinary printer names (case-insensitive).",
                ));
            }
        }
        let mut files = BTreeSet::new();
        let mut bytes = 0usize;
        for doc in &self.documents {
            bytes = bytes
                .checked_add(doc.text.len())
                .ok_or_else(|| invalid("Document size limit."))?;
            if !file_name(&doc.path)
                || !bounded(&doc.title, 80)
                || doc.text.len() > 16384
                || doc
                    .text
                    .chars()
                    .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
                || !(1..=100).contains(&doc.pages)
                || !files.insert(doc.path.as_str())
            {
                return Err(invalid(
                    "Use distinct supplied virtual documents with ordinary paths, bounded text and 1–100 modeled pages.",
                ));
            }
        }
        if bytes > 65536 {
            return Err(invalid("Supplied documents exceed 64 KiB."));
        }
        Ok(())
    }
    pub fn document(&self, path: &str) -> Result<&Document> {
        self.documents
            .iter()
            .find(|d| d.path == path)
            .ok_or_else(|| {
                invalid(
                    "That file is not in the supplied virtual documents; no host file is opened.",
                )
            })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Printer {
    pub name: QueueName,
    pub accepting: bool,
    pub enabled: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub rejection_reason: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub stopped_reason: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Pending,
    Held,
    Completed,
    Canceled,
}
impl JobState {
    pub fn active(self) -> bool {
        matches!(self, Self::Pending | Self::Held)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    pub id: u32,
    pub printer: QueueName,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub document: String,
    pub copies: u16,
    pub state: JobState,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Printer,8>")]
    pub printers: Vec<Printer>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Job,64>")]
    pub jobs: Vec<Job>,
    pub local_default: Option<QueueName>,
    pub next_id: u32,
}
impl World {
    pub fn validate(&self, f: &Facts) -> Result<()> {
        f.validate()?;
        if self.printers.len() != f.printers.len() || self.jobs.len() > MAX_JOBS {
            return Err(invalid(
                "Printer identities or job capacity disagree with the supplied workspace.",
            ));
        }
        let mut names = BTreeSet::new();
        for p in &self.printers {
            if !f.printers.contains(&p.name)
                || !names.insert(p.name.0.to_ascii_lowercase())
                || p.rejection_reason.len() > 120
                || p.stopped_reason.len() > 120
                || p.rejection_reason.chars().any(char::is_control)
                || p.stopped_reason.chars().any(char::is_control)
            {
                return Err(invalid("Invalid supplied printer identity/status."));
            }
        }
        if let Some(default) = &self.local_default {
            self.printer(&default.0)?;
        }
        let mut ids = BTreeSet::new();
        for j in &self.jobs {
            self.printer(&j.printer.0)?;
            f.document(&j.document)?;
            if j.id == 0
                || j.id >= self.next_id
                || !ids.insert(j.id)
                || !(1..=20).contains(&j.copies)
            {
                return Err(invalid("Invalid or duplicated request ID/copy count."));
            }
        }
        if self.next_id == 0 || self.next_id > 1_000_000 {
            return Err(invalid("Request number limit reached."));
        }
        Ok(())
    }
    pub fn printer(&self, name: &str) -> Result<&Printer> {
        self.printers
            .iter()
            .find(|p| p.name.0.eq_ignore_ascii_case(name))
            .ok_or_else(|| {
                invalid("Unknown supplied printer. Use lpstat to inspect the available names.")
            })
    }
    fn printer_mut(&mut self, name: &str) -> Result<&mut Printer> {
        self.printers
            .iter_mut()
            .find(|p| p.name.0.eq_ignore_ascii_case(name))
            .ok_or_else(|| invalid("Unknown supplied printer."))
    }
    pub fn request(&self, raw: &str) -> Result<&Job> {
        let number = if let Ok(id) = raw.parse::<u32>() {
            id
        } else {
            let (name, id) = raw.rsplit_once('-').ok_or_else(|| {
                invalid("Use the request number or complete printer-number ID shown by lpstat.")
            })?;
            let id = id
                .parse::<u32>()
                .map_err(|_| invalid("Use a numeric request ID."))?;
            let job = self
                .jobs
                .iter()
                .find(|j| j.id == id)
                .ok_or_else(|| invalid("No such simulated request."))?;
            if !job.printer.0.eq_ignore_ascii_case(name) {
                return Err(invalid("That request is not on the named printer."));
            }
            id
        };
        self.jobs
            .iter()
            .find(|j| j.id == number)
            .ok_or_else(|| invalid("No such simulated request."))
    }
    /// Explicit deterministic scheduler, invoked by supported mutations, never a timer.
    fn dispatch(&mut self, printer: &str) {
        let enabled = self
            .printers
            .iter()
            .any(|p| p.name.0.eq_ignore_ascii_case(printer) && p.enabled);
        if enabled {
            for job in &mut self.jobs {
                if job.printer.0.eq_ignore_ascii_case(printer) && job.state == JobState::Pending {
                    job.state = JobState::Completed;
                }
            }
        }
    }
    pub fn set_accepting(&mut self, name: &str, value: bool, reason: &str) -> Result<()> {
        if reason.len() > 120 || reason.chars().any(char::is_control) {
            return Err(invalid("Use a short single-line reason."));
        }
        let p = self.printer_mut(name)?;
        p.accepting = value;
        p.rejection_reason = if value { String::new() } else { reason.into() };
        Ok(())
    }
    pub fn set_enabled(&mut self, name: &str, value: bool, reason: &str) -> Result<()> {
        if reason.len() > 120 || reason.chars().any(char::is_control) {
            return Err(invalid("Use a short single-line reason."));
        }
        let p = self.printer_mut(name)?;
        p.enabled = value;
        p.stopped_reason = if value { String::new() } else { reason.into() };
        if value {
            self.dispatch(name);
        }
        Ok(())
    }
    pub fn set_default(&mut self, name: &str) -> Result<()> {
        let canonical = self.printer(name)?.name.clone();
        self.local_default = Some(canonical);
        Ok(())
    }
    pub fn submit(
        &mut self,
        f: &Facts,
        printer: Option<&str>,
        document: &str,
        copies: u16,
        held: bool,
    ) -> Result<u32> {
        f.document(document)?;
        if self.jobs.len() >= MAX_JOBS || self.next_id >= 1_000_000 {
            return Err(invalid(
                "Simulated job history is full; reset only this attempt to start over.",
            ));
        }
        if !(1..=20).contains(&copies) {
            return Err(invalid("This simulator supports 1–20 copies per request."));
        }
        let name = printer
            .map(str::to_owned)
            .or_else(|| self.local_default.as_ref().map(|n| n.0.clone()))
            .ok_or_else(|| {
                invalid(
                    "No default destination. Select a printer explicitly or set a local default.",
                )
            })?;
        let p = self.printer(&name)?;
        if !p.accepting {
            return Err(invalid(format!(
                "{} is rejecting new jobs: {}. Accepting jobs and processing them are separate settings.",
                p.name.0, p.rejection_reason
            )));
        }
        let name = p.name.clone();
        let id = self.next_id;
        self.next_id += 1;
        self.jobs.push(Job {
            id,
            printer: name.clone(),
            document: document.into(),
            copies,
            state: if held {
                JobState::Held
            } else {
                JobState::Pending
            },
        });
        self.dispatch(&name.0);
        Ok(id)
    }
    pub fn hold(&mut self, request: &str, held: bool) -> Result<()> {
        let job = self.request(request)?;
        if !job.state.active() {
            return Err(invalid(
                "That request has already finished; it cannot be held or resumed in this subset.",
            ));
        }
        let id = job.id;
        let name = job.printer.0.clone();
        self.jobs.iter_mut().find(|j| j.id == id).unwrap().state = if held {
            JobState::Held
        } else {
            JobState::Pending
        };
        if !held {
            self.dispatch(&name);
        }
        Ok(())
    }
    pub fn cancel(&mut self, request: &str) -> Result<()> {
        let job = self.request(request)?;
        if !job.state.active() {
            return Err(invalid(
                "Only pending or held requests can be canceled in this subset.",
            ));
        }
        let id = job.id;
        self.jobs.iter_mut().find(|j| j.id == id).unwrap().state = JobState::Canceled;
        Ok(())
    }
    pub fn completed_copies(&self, document: &str, printer: &str) -> u32 {
        self.jobs
            .iter()
            .filter(|j| {
                j.document == document
                    && j.printer.0.eq_ignore_ascii_case(printer)
                    && j.state == JobState::Completed
            })
            .map(|j| u32::from(j.copies))
            .sum()
    }
}

mod bsd;
pub mod command;
mod literal;
#[cfg(test)]
#[path = "print_queues/tests.rs"]
mod tests;
