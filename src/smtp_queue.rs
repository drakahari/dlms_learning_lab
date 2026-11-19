//! A bounded sample queue, saved/active text and delivery records. No MTA or host I/O.
use crate::{
    smtp_config::{Config, ErrorKind, Fault, MAX_LINES, MAX_TEXT},
    smtp_routing::{self as routing, Outcome, Recipient, Trace},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_MESSAGES: usize = 16;
pub const MAX_SOURCE: usize = 256 * 1024;
pub const MAX_RUNTIME: usize = 384 * 1024;
fn invalid(message: impl Into<String>) -> Fault {
    Fault {
        kind: ErrorKind::Invalid,
        message: message.into(),
    }
}
fn limit(message: impl Into<String>) -> Fault {
    Fault {
        kind: ErrorKind::Limit,
        message: message.into(),
    }
}
fn within<T: Serialize>(value: &T, cap: usize) -> bool {
    struct Counter(usize, usize);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .filter(|n| *n <= self.1)
                .ok_or_else(|| std::io::Error::other("Sample-mail encoding limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter(0, cap), value).is_ok()
}
fn file_shape(text: &str) -> bool {
    text.len() <= MAX_TEXT && text.lines().count() <= MAX_LINES
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub entry: String,
    pub sender: Option<String>,
    pub recipient: String,
    pub subject: String,
    pub body: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub world: routing::Facts,
    pub initial_files: BTreeMap<String, String>,
    pub initial_active: BTreeMap<String, String>,
    /// Immutable exercise inputs. Bodies are never scripts or editable queue files.
    pub samples: BTreeMap<String, Sample>,
}
impl Facts {
    pub fn validate(&self) -> Result<(), Fault> {
        if !within(self, MAX_SOURCE) || !(1..=MAX_MESSAGES).contains(&self.samples.len()) {
            return Err(limit(
                "Supply one to16bounded sample messages within256KiB of source data.",
            ));
        }
        self.world.validate()?;
        if self.initial_files.keys().ne(self.world.nodes.keys())
            || self.initial_active.keys().ne(self.world.nodes.keys())
            || self.initial_files.values().any(|t| !file_shape(t))
        {
            return Err(invalid(
                "Supply one bounded saved and active configuration for each represented server.",
            ));
        }
        for text in self.initial_active.values() {
            Config::parse(text)?;
        }
        for (id, sample) in &self.samples {
            if !crate::local_mail::name(id)
                || *id != id.to_ascii_lowercase()
                || !self.world.nodes.contains_key(&sample.entry)
                || sample.subject.len() > 160
                || sample.subject.chars().any(char::is_control)
                || sample.body.len() > 2048
                || sample.body.contains('\0')
            {
                return Err(invalid(
                    "Sample messages need stable simple IDs, a supplied queue owner, short single-line subjects and bodies within2KiB.",
                ));
            }
            Recipient::parse(&sample.recipient)?;
            if let Some(sender) = &sample.sender {
                Recipient::parse(sender)?;
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Status {
    Queued,
    Deferred {
        reason: String,
    },
    Failed {
        reason: String,
    },
    Delivered {
        node: String,
        mailboxes: BTreeSet<String>,
    },
    Partial {
        node: String,
        mailboxes: BTreeSet<String>,
        failures: BTreeSet<String>,
    },
}
impl Status {
    pub fn pending(&self) -> bool {
        matches!(self, Self::Queued | Self::Deferred { .. })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    /// Deterministic opaque sample identifier, not a host Postfix queue ID.
    pub queue_id: String,
    pub status: Status,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub version: u32,
    pub files: BTreeMap<String, String>,
    pub active: BTreeMap<String, String>,
    pub messages: BTreeMap<String, Message>,
}
fn messages(f: &Facts) -> BTreeMap<String, Message> {
    f.samples
        .keys()
        .enumerate()
        .map(|(n, id)| {
            (
                id.clone(),
                Message {
                    queue_id: format!("{:08X}", 0xA0000000u32 + n as u32),
                    status: Status::Queued,
                },
            )
        })
        .collect()
}
impl Runtime {
    pub fn new(f: &Facts) -> Result<Self, Fault> {
        f.validate()?;
        Ok(Self {
            version: 1,
            files: f.initial_files.clone(),
            active: f.initial_active.clone(),
            messages: messages(f),
        })
    }
    pub fn validate(&self, f: &Facts) -> Result<(), Fault> {
        f.validate()?;
        if !within(self, MAX_RUNTIME) {
            return Err(limit("The sample-mail runtime exceeds384KiB."));
        }
        if self.version != 1
            || self.files.keys().ne(f.world.nodes.keys())
            || self.active.keys().ne(f.world.nodes.keys())
            || self.messages.keys().ne(f.samples.keys())
            || self.files.values().any(|s| !file_shape(s))
        {
            return Err(invalid(
                "Sample-mail state must retain its version, supplied servers, configuration files and every original sample.",
            ));
        }
        for text in self.active.values() {
            Config::parse(text)?;
        }
        let original = messages(f);
        for (id, message) in &self.messages {
            if message.queue_id != original[id].queue_id {
                return Err(invalid("Sample queue identifiers belong to this exercise."));
            }
            match &message.status {
                Status::Deferred { reason } | Status::Failed { reason }
                    if reason.len() > 1024 || reason.contains('\0') =>
                {
                    return Err(invalid("Keep delivery diagnostics bounded and readable."));
                }
                Status::Delivered { node, mailboxes }
                | Status::Partial {
                    node, mailboxes, ..
                } => {
                    let n = f.world.nodes.get(node).ok_or_else(|| {
                        invalid("A delivery record needs a supplied recipient server.")
                    })?;
                    if mailboxes.is_empty()
                        || mailboxes.len() > 8
                        || mailboxes.iter().any(|u| !n.users.contains(u))
                    {
                        return Err(invalid(
                            "A delivery record needs actual supplied local mailboxes.",
                        ));
                    }
                    if let Status::Partial { failures, .. } = &message.status
                        && (failures.is_empty()
                            || failures.len() > 128
                            || failures.iter().any(|s| s.len() > 512 || s.contains('\0')))
                    {
                        return Err(invalid("Partial delivery needs bounded explicit failures."));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    pub fn save(&mut self, f: &Facts, node: &str, text: &str) -> Result<(), Fault> {
        self.validate(f)?;
        if !f.world.nodes.contains_key(node) {
            return Err(invalid(
                "Choose a supplied server before saving configuration.",
            ));
        }
        if !file_shape(text) {
            return Err(limit(
                "Keep this simulated configuration within8KiB and96lines.",
            ));
        }
        // Saving a malformed virtual file is possible; reload then fails without replacing active text.
        self.files.insert(node.into(), text.into());
        Ok(())
    }
    pub fn reload(&mut self, f: &Facts, node: &str) -> Result<(), Fault> {
        self.validate(f)?;
        let text = self
            .files
            .get(node)
            .ok_or_else(|| invalid("Choose a supplied server before reloading."))?;
        Config::parse(text)?;
        self.active.insert(node.into(), text.clone());
        Ok(())
    }
    pub fn pending(&self, node: &str, f: &Facts) -> Result<Vec<&str>, Fault> {
        self.validate(f)?;
        if !f.world.nodes.contains_key(node) {
            return Err(invalid("Choose a supplied queue owner."));
        }
        Ok(self
            .messages
            .iter()
            .filter(|(id, m)| f.samples[*id].entry == node && m.status.pending())
            .map(|(_, m)| m.queue_id.as_str())
            .collect())
    }
    /// One bounded retry per pending sample owned by this server, never a timer loop.
    pub fn flush(&mut self, f: &Facts, node: &str) -> Result<Vec<(String, Trace)>, Fault> {
        self.validate(f)?;
        if !f.world.nodes.contains_key(node) {
            return Err(invalid("Choose a supplied queue owner."));
        }
        let prepared = f.world.prepare(&self.active)?;
        let mut observations = vec![];
        for (id, message) in &mut self.messages {
            let sample = &f.samples[id];
            if sample.entry != node || !message.status.pending() {
                continue;
            }
            let trace = prepared.queued(node, &sample.recipient)?;
            message.status=match &trace.outcome {
                Outcome::Local{node,mailboxes,failures} if failures.is_empty()&&!mailboxes.is_empty()=>Status::Delivered{node:node.clone(),mailboxes:mailboxes.clone()},
                Outcome::Local{node,mailboxes,failures} if !mailboxes.is_empty()=>Status::Partial{node:node.clone(),mailboxes:mailboxes.clone(),failures:failures.clone()},
                Outcome::Local{..}=>Status::Failed{reason:"The represented local destination has no successful recipient. Inspect its local-domain role and supplied recipients.".into()},
                Outcome::Rejected{node}=>Status::Failed{reason:format!("{node} refused relay permission for this already queued sample.")},
                Outcome::Deferred{node}=>Status::Deferred{reason:format!("{node} temporarily deferred relay permission.")},
                Outcome::Unavailable{target,port}=>Status::Deferred{reason:format!("No available supplied endpoint at {target}:{port}.")},
                Outcome::NoRoute{node}=>Status::Deferred{reason:format!("{node} has no supplied literal next hop. No DNS/MX lookup is performed.")},
                Outcome::ConfigurationProblem{node,error}=>Status::Deferred{reason:format!("{node}: {}",error.message)},
                Outcome::Loop{repeats,..}=>Status::Deferred{reason:format!("The model relay path repeats {repeats}. This finite diagnostic is not a real SMTP bounce or queue-timer result.")},
                Outcome::Limit=>Status::Deferred{reason:"The route observation reached its finite hop limit.".into()},
            };
            observations.push((id.clone(), trace));
        }
        Ok(observations)
    }
    /// Simulator-only replay helper: restores sample mail, retaining both file sets.
    /// It is not a Postfix command and never operates on another lesson's work.
    pub fn reset_samples(&mut self, f: &Facts) -> Result<(), Fault> {
        self.validate(f)?;
        self.messages = messages(f);
        Ok(())
    }
}
