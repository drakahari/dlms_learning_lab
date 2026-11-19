//! Health and short self-tests over a supplied virtual ATA inventory.
pub mod command;

use self::command::{Command, Operation};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt::Write};

type Result<T> = std::result::Result<T, String>;
const DISK_LIMIT: usize = 8;
const LOG_LIMIT: usize = 8;

fn disks<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Disk>, D::Error> {
    crate::cron_workspace::sequence::<_, Disk, DISK_LIMIT>(d)
}

fn history<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<LogEntry>, D::Error> {
    crate::cron_workspace::sequence::<_, LogEntry, LOG_LIMIT>(d)
}

fn states<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<DiskState>, D::Error> {
    crate::cron_workspace::sequence::<_, DiskState, DISK_LIMIT>(d)
}

fn short_selector(name: &str) -> bool {
    matches!(name.as_bytes(), [b'/', b'd', b'e', b'v', b'/', b's', b'd', letter] if letter.is_ascii_lowercase())
}

fn stable_selector(name: &str) -> bool {
    name.strip_prefix("/dev/disk/by-id/ata-").is_some_and(|s| {
        !s.is_empty()
            && s.len() <= 72
            && s.as_bytes()[0].is_ascii_alphanumeric()
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
    })
}

pub fn valid_selector(name: &str) -> bool {
    short_selector(name) || stable_selector(name)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Health {
    Passed,
    Failed,
}

impl Health {
    fn display(self) -> &'static str {
        match self {
            Self::Passed => "PASSED",
            Self::Failed => "FAILED!",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SelfTestResult {
    Completed,
    ReadFailure { first_lba: u64 },
}

impl SelfTestResult {
    fn text(self) -> &'static str {
        match self {
            Self::Completed => "Completed without error",
            Self::ReadFailure { .. } => "Completed: read failure",
        }
    }

    fn lba(self) -> String {
        match self {
            Self::Completed => "-".into(),
            Self::ReadFailure { first_lba } => first_lba.to_string(),
        }
    }

    fn failed(self) -> bool {
        matches!(self, Self::ReadFailure { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogEntry {
    pub hours: u16,
    pub result: SelfTestResult,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Disk {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub model: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub serial: String,
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub stable: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8>")]
    pub current_name: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8>")]
    pub maintenance_name: String,
    pub health: Health,
    pub hours: u16,
    pub short_test_minutes: u8,
    pub new_short_result: SelfTestResult,
    #[serde(deserialize_with = "history")]
    pub log: Vec<LogEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Current,
    Maintenance,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    #[serde(deserialize_with = "disks")]
    pub disks: Vec<Disk>,
}

impl Inventory {
    pub fn validate(&self) -> Result<()> {
        if self.disks.is_empty() || self.disks.len() > DISK_LIMIT {
            return Err("Supply one to eight virtual ATA disks.".into());
        }
        let mut stable = BTreeSet::new();
        let mut current = BTreeSet::new();
        let mut future = BTreeSet::new();
        let mut serial = BTreeSet::new();
        for disk in &self.disks {
            if !stable_selector(&disk.stable)
                || !stable.insert(&disk.stable)
                || !short_selector(&disk.current_name)
                || !current.insert(&disk.current_name)
                || !short_selector(&disk.maintenance_name)
                || !future.insert(&disk.maintenance_name)
                || !valid_label(&disk.model, 48)
                || !valid_label(&disk.serial, 32)
                || !serial.insert(&disk.serial)
                || !(1..=9).contains(&disk.short_test_minutes)
                || disk.log.len() > LOG_LIMIT
                || disk.log.iter().any(|entry| entry.hours > disk.hours)
                || disk
                    .log
                    .windows(2)
                    .any(|pair| pair[0].hours < pair[1].hours)
            {
                return Err("Review disk identities, unique paths in each layout, test duration, and newest-first history.".into());
            }
        }
        Ok(())
    }

    pub fn find(&self, name: &str, layout: Layout) -> Option<(usize, &Disk)> {
        self.disks.iter().enumerate().find(|(_, disk)| {
            name == disk.stable
                || name
                    == match layout {
                        Layout::Current => &disk.current_name,
                        Layout::Maintenance => &disk.maintenance_name,
                    }
        })
    }
}

fn valid_label(text: &str, max: usize) -> bool {
    !text.trim().is_empty()
        && text.len() <= max
        && text.is_ascii()
        && !text.bytes().any(|b| b.is_ascii_control())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiskState {
    pub remaining_minutes: Option<u8>,
    pub last_current: Option<LogEntry>,
    pub started: u8,
    pub completed: u8,
    #[serde(deserialize_with = "history")]
    pub log: Vec<LogEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub minutes: u16,
    #[serde(deserialize_with = "states")]
    pub disks: Vec<DiskState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub status: u8,
    pub text: String,
}

impl State {
    pub fn new(inventory: &Inventory) -> Self {
        Self {
            minutes: 0,
            disks: inventory
                .disks
                .iter()
                .map(|disk| DiskState {
                    remaining_minutes: None,
                    last_current: None,
                    started: 0,
                    completed: 0,
                    log: disk.log.clone(),
                })
                .collect(),
        }
    }

    pub fn validate(&self, inventory: &Inventory) -> Result<()> {
        if self.disks.len() != inventory.disks.len() {
            return Err("The saved test state does not match the supplied disks.".into());
        }
        for (state, disk) in self.disks.iter().zip(&inventory.disks) {
            let added = usize::from(state.completed).min(LOG_LIMIT);
            let retained = &disk.log[..disk.log.len().min(LOG_LIMIT - added)];
            let expected_started = state
                .completed
                .checked_add(u8::from(state.remaining_minutes.is_some()));
            let expected_current = LogEntry {
                hours: disk.hours.saturating_add(self.minutes / 60),
                result: disk.new_short_result,
            };
            if expected_started != Some(state.started)
                || state
                    .remaining_minutes
                    .is_some_and(|v| v == 0 || v > disk.short_test_minutes)
                || state.completed == 0 && (state.last_current.is_some() || state.log != disk.log)
                || state.completed > 0
                    && state.last_current.as_ref().is_none_or(|v| {
                        v.result != expected_current.result
                            || v.hours < disk.hours
                            || v.hours > expected_current.hours
                    })
                || state.last_current.is_some() && state.last_current.as_ref() != state.log.first()
                || state.log.len() != added + retained.len()
                || state.log.get(added..) != Some(retained)
                || state.log.iter().take(added).any(|e| {
                    e.result != disk.new_short_result
                        || e.hours < disk.hours
                        || e.hours > expected_current.hours
                })
                || state
                    .log
                    .windows(2)
                    .any(|pair| pair[0].hours < pair[1].hours)
            {
                return Err("The saved test clock and disk history are inconsistent.".into());
            }
        }
        Ok(())
    }

    pub fn advance(&mut self, inventory: &Inventory) -> Vec<String> {
        self.minutes = self.minutes.saturating_add(1);
        let mut events = Vec::new();
        for (state, disk) in self.disks.iter_mut().zip(&inventory.disks) {
            match state.remaining_minutes {
                None => {}
                Some(n) if n > 1 => state.remaining_minutes = Some(n - 1),
                Some(_) => {
                    state.remaining_minutes = None;
                    state.completed += 1;
                    let entry = LogEntry {
                        hours: disk.hours.saturating_add(self.minutes / 60),
                        result: disk.new_short_result,
                    };
                    state.last_current = Some(entry.clone());
                    state.log.insert(0, entry);
                    state.log.truncate(LOG_LIMIT);
                    events.push(format!(
                        "The short test of {} finished. Inspect its self-test log.",
                        disk.serial
                    ));
                }
            }
        }
        events
    }

    pub fn execute(&mut self, inventory: &Inventory, command: Command) -> Result<Output> {
        let Command::Device { name, operation } = command else {
            return Ok(Output {
                status: 0,
                text: command::HELP.into(),
            });
        };
        let (index, disk) = inventory
            .find(&name, Layout::Current)
            .ok_or_else(|| format!("{name} is not one of the virtual disks supplied here."))?;
        let state = &mut self.disks[index];
        let mut status = 0;
        let text = match operation {
            Operation::Info => format!(
                "Device Model: {}\nSerial Number: {}\nSMART support is: Enabled",
                disk.model, disk.serial
            ),
            Operation::Health => {
                if disk.health == Health::Failed {
                    status = 1 << 3;
                }
                format!(
                    "SMART overall-health self-assessment test result: {}",
                    disk.health.display()
                )
            }
            Operation::Capabilities => {
                let current = state.remaining_minutes.map_or_else(
                    || "No self-test is in progress.".into(),
                    |left| format!("Self-test routine in progress. About {left} simulated minute(s) remain."),
                );
                format!(
                    "Self-test execution status: {current}\nShort self-test routine recommended polling time: {} minute(s).",
                    disk.short_test_minutes
                )
            }
            Operation::SelfTestLog => {
                let mut text = String::from(
                    "SMART Self-test log\nNum  Test_Description  Status                  LifeTime(hours)  LBA_of_first_error",
                );
                if state.log.is_empty() {
                    text.push_str("\nNo self-tests logged.");
                }
                for (number, entry) in state.log.iter().enumerate() {
                    if entry.result.failed() {
                        status |= 1 << 7;
                    }
                    let _ = write!(
                        text,
                        "\n#{:<3} Short offline     {:24} {:>15}  {}",
                        number + 1,
                        entry.result.text(),
                        entry.hours,
                        entry.result.lba()
                    );
                }
                text
            }
            Operation::StartShort => {
                if state.remaining_minutes.is_some() {
                    return Err("A short self-test is already running on this disk. Inspect its status and advance virtual time.".into());
                }
                state.started = state
                    .started
                    .checked_add(1)
                    .ok_or("This practice history reached its test limit. Reset to begin again.")?;
                state.remaining_minutes = Some(disk.short_test_minutes);
                format!(
                    "Testing has begun. Short self-test polling time: {} minute(s). Use Advance virtual time to observe it.",
                    disk.short_test_minutes
                )
            }
        };
        Ok(Output { status, text })
    }
}

pub mod monitor;

pub mod session;
pub mod workspace;

pub mod source;
