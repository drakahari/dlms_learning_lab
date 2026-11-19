//! Bounded local notice capability; no curriculum registered by this module.
//! No host sessions, files, clocks, messages or restart operations.
use serde::{Deserialize, Serialize};
pub mod bulletin;
pub mod command;
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_TEXT: usize = 4096;
pub const MAX_EVENTS: usize = 64;
type Result<T> = std::result::Result<T, String>;
fn text(value: &str) -> bool {
    value.len() <= MAX_TEXT
        && value
            .chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\t'))
}
fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 48
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
fn terminal(value: &str) -> bool {
    let suffix = value
        .strip_prefix("pts/")
        .or_else(|| value.strip_prefix("tty"));
    suffix.is_some_and(|n| !n.is_empty() && n.len() <= 4 && n.bytes().all(|b| b.is_ascii_digit()))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Login {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub user: String,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub tty: String,
    pub since_minute: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Document {
    Issue,
    Motd,
}
impl Document {
    pub fn path(self) -> &'static str {
        match self {
            Self::Issue => "/etc/issue",
            Self::Motd => "/etc/motd",
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Files {
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub issue: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub motd: String,
}
impl Files {
    pub fn validate(&self) -> Result<()> {
        if text(&self.issue) && !self.issue.contains('\\') && text(&self.motd) {
            Ok(())
        } else {
            Err("Use bounded plain-text notices with no terminal control sequences.".into())
        }
    }
    pub fn get(&self, document: Document) -> &str {
        match document {
            Document::Issue => &self.issue,
            Document::Motd => &self.motd,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub host: String,
    /// UTC midnight for the supplied recording, never the desktop clock.
    pub date: crate::cron::Moment,
    pub minute: u32,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Login,8>")]
    pub logins: Vec<Login>,
    pub initial: Files,
}
impl Facts {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.date.validate().map_err(|e| e.to_string())?;
        // The bounded clock ends before day three. A valid midnight date
        // needs only one further calendar day within Moment's2099 ceiling.
        // Do not walk2879minutes on every state validation/UI frame.
        if self.date.year == 2099 && self.date.month == 12 && self.date.day == 31 {
            return Err("The recording needs two calendar days within2000–2099.".into());
        }
        if self.date.hour != 0 || self.date.minute != 0 {
            return Err("The supplied recording date must start at UTC midnight.".into());
        }
        if self.version != 1
            || !name(&self.host)
            || self.minute > 1439
            || self.logins.is_empty()
            || self.logins.len() > 8
            || self
                .logins
                .iter()
                .any(|l| !name(&l.user) || !terminal(&l.tty) || l.since_minute > self.minute)
            || self
                .logins
                .iter()
                .map(|l| &l.tty)
                .collect::<BTreeSet<_>>()
                .len()
                != self.logins.len()
        {
            return Err(
                "Supply one finite recorded day and distinct local terminal sessions.".into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Save {
        document: Document,
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
    Broadcast {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
    Schedule {
        after_minutes: u32,
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
    Cancel {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
    /// Explicit simulator activity, never taught as a Linux command.
    Advance { minutes: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delivery {
    pub minute: u32,
    pub body: String,
    pub terminals: BTreeSet<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Restart {
    pub at: u32,
    pub body: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
    pub minute: u32,
    pub files: Files,
    pub deliveries: Vec<Delivery>,
    pub restart: Option<Restart>,
    pub stopped: bool,
}
impl World {
    pub fn initial(facts: &Facts) -> Result<Self> {
        facts.validate()?;
        Ok(Self {
            minute: facts.minute,
            files: facts.initial.clone(),
            deliveries: vec![],
            restart: None,
            stopped: false,
        })
    }
    pub fn replay(facts: &Facts, actions: &[Action]) -> Result<Self> {
        if actions.len() > MAX_EVENTS {
            return Err("The finite notice activity supports at most 64 saved actions; reset this attempt to start again.".into());
        }
        let mut world = Self::initial(facts)?;
        for action in actions {
            world.apply(facts, action)?;
        }
        Ok(world)
    }
    pub fn ordinary_logins_blocked(&self) -> bool {
        self.stopped
            || self
                .restart
                .as_ref()
                .is_some_and(|r| r.at.saturating_sub(self.minute) <= 5)
    }
    fn broadcast(&mut self, facts: &Facts, body: &str) -> Result<()> {
        if !text(body) || body.trim().is_empty() {
            return Err("Write a nonempty plain-text notice before broadcasting.".into());
        }
        if self.deliveries.len() >= MAX_EVENTS {
            return Err("The simulated delivery history is full.".into());
        }
        self.deliveries.push(Delivery {
            minute: self.minute,
            body: body.into(),
            terminals: facts.logins.iter().map(|l| l.tty.clone()).collect(),
        });
        Ok(())
    }
    /// Called only on an owned clone; refusal never commits partial changes.
    pub fn apply(&mut self, facts: &Facts, action: &Action) -> Result<()> {
        facts.validate()?;
        if self.stopped {
            return Err("The recorded host reached its simulated restart. Reset this attempt to explore again; no actual machine restarted.".into());
        }
        match action {
            Action::Save {
                document,
                text: value,
            } => {
                if !text(value) {
                    return Err(
                        "Keep this simulated notice within 4 KiB, with no control sequences."
                            .into(),
                    );
                }
                if *document == Document::Issue && value.contains('\\') {
                    return Err("Getty backslash substitutions in issue are outside this plain-text notice subset.".into());
                }
                match document {
                    Document::Issue => self.files.issue = value.clone(),
                    Document::Motd => self.files.motd = value.clone(),
                }
            }
            Action::Broadcast { text } => self.broadcast(facts, text)?,
            Action::Schedule {
                after_minutes,
                text,
            } => {
                if !(1..=1440).contains(after_minutes)
                    || self
                        .minute
                        .checked_add(*after_minutes)
                        .is_none_or(|m| m > 2879)
                {
                    return Err("This subset schedules an explicit future restart from 1 to 1440 simulated minutes; now, past times and other clock formats are outside scope.".into());
                }
                // Do not silently approximate whether a host accepts replacing
                // an existing schedule: require cancellation in this subset.
                if self.restart.is_some() {
                    return Err("Cancel the existing scheduled restart before scheduling another in this simulation.".into());
                }
                self.broadcast(facts, text)?;
                self.restart = Some(Restart {
                    at: self.minute + after_minutes,
                    body: text.clone(),
                });
            }
            Action::Cancel { text } => {
                if self.restart.is_none() {
                    return Err("There is no pending simulated restart to cancel.".into());
                }
                self.broadcast(facts, text)?;
                self.restart = None;
            }
            Action::Advance { minutes } => {
                if !(1..=1440).contains(minutes)
                    || self.minute.checked_add(*minutes).is_none_or(|m| m > 2879)
                {
                    return Err(
                        "Advance only 1 to 1440 minutes within the supplied two-day clock.".into(),
                    );
                }
                let next = self.minute + minutes;
                if let Some(restart) = &self.restart
                    && next >= restart.at
                {
                    self.minute = restart.at;
                    self.stopped = true;
                    return Ok(());
                }
                self.minute = next;
            }
        }
        Ok(())
    }
    pub fn receipt_by_terminal(&self) -> BTreeMap<String, Vec<&Delivery>> {
        let mut result = BTreeMap::new();
        for delivery in &self.deliveries {
            for terminal in &delivery.terminals {
                result
                    .entry(terminal.clone())
                    .or_insert_with(Vec::new)
                    .push(delivery);
            }
        }
        result
    }
}
