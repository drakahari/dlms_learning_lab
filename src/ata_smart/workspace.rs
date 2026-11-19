//! Grade a current diagnostic result or monitoring policy directly from virtual state.
use super::{Inventory, Layout, Result, SelfTestResult, State, command, monitor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub primary_serial: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Monitoring {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub primary_serial: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub auxiliary_serial: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub initial_config: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Assignment {
    Diagnostic(Diagnostic),
    Monitoring(Monitoring),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub inventory: Inventory,
    pub goal: Assignment,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    #[default]
    Unset,
    Completed,
    ReadFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ticket {
    pub disposition: Disposition,
    #[serde(deserialize_with = "crate::cron::text::<_,24>")]
    pub first_lba: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub label: &'static str,
    pub passed: bool,
    pub detail: String,
}

impl Task {
    pub fn validate(&self) -> Result<()> {
        self.inventory.validate()?;
        let has = |serial: &str| {
            self.inventory
                .disks
                .iter()
                .any(|disk| disk.serial == serial)
        };
        match &self.goal {
            Assignment::Diagnostic(goal) if has(&goal.primary_serial) => Ok(()),
            Assignment::Monitoring(goal)
                if goal.primary_serial != goal.auxiliary_serial
                    && has(&goal.primary_serial)
                    && has(&goal.auxiliary_serial) =>
            {
                monitor::parse(&goal.initial_config)?;
                let checks = self.monitoring_checks(goal, &goal.initial_config);
                if checks.iter().all(|result| result.passed) {
                    Err("Supply an initial monitoring fault that the learner can correct.".into())
                } else {
                    Ok(())
                }
            }
            _ => {
                Err("Use supplied and distinct disk serial numbers for the required roles.".into())
            }
        }
    }

    pub fn diagnostic_checks(
        &self,
        goal: &Diagnostic,
        state: &State,
        ticket: &Ticket,
    ) -> Vec<Check> {
        let Some((index, _)) = self
            .inventory
            .disks
            .iter()
            .enumerate()
            .find(|(_, d)| d.serial == goal.primary_serial)
        else {
            return vec![Check {
                label: "Disk identity",
                passed: false,
                detail: "The required disk is absent.".into(),
            }];
        };
        let target = &state.disks[index];
        let done = target.remaining_minutes.is_none() && target.last_current.is_some();
        let lba = ticket.first_lba.trim();
        let decimal_lba = lba.bytes().all(|b| b.is_ascii_digit());
        let wanted_lba = if decimal_lba {
            lba.parse::<u64>().ok()
        } else {
            None
        };
        let evidence = target
            .last_current
            .as_ref()
            .is_some_and(|entry| match entry.result {
                SelfTestResult::Completed => {
                    ticket.disposition == Disposition::Completed
                        && ticket.first_lba.trim().is_empty()
                }
                SelfTestResult::ReadFailure { first_lba } => {
                    ticket.disposition == Disposition::ReadFailure && wanted_lba == Some(first_lba)
                }
            });
        let other = self
            .inventory
            .disks
            .iter()
            .zip(&state.disks)
            .enumerate()
            .find(|(i, (_, disk))| *i != index && disk.started > 0);
        vec![
            Check {
                label: "Completed short test",
                passed: done,
                detail: if target.remaining_minutes.is_some() {
                    "A test is running on the required disk. Advance virtual time and then read the completed log.".into()
                } else if done {
                    "A fresh short test completed on the disk in the ticket.".into()
                } else {
                    "Find the disk named in the ticket and start its short test. The older log is only background evidence.".into()
                },
            },
            Check {
                label: "Maintenance ticket",
                passed: done && evidence,
                detail: if !done {
                    "Read the fresh completed self-test before recording its result.".into()
                } else if ticket.disposition == Disposition::Unset {
                    "Record the result from the newest self-test line. Use its first-error LBA only if that test reports a read failure.".into()
                } else if evidence {
                    "The ticket matches the latest completed result and its error location.".into()
                } else if ticket.disposition == Disposition::ReadFailure && lba.is_empty() {
                    "Enter the first-error block address from the newest self-test line.".into()
                } else if ticket.disposition == Disposition::ReadFailure && wanted_lba.is_none() {
                    "Use a decimal block address made of digits. Copy the value from the newest test's LBA column.".into()
                } else if target.last_current.as_ref().is_some_and(|entry| matches!(entry.result, SelfTestResult::Completed)) {
                    "The latest short test completed without a read failure. An older log entry can still show a failure.".into()
                } else {
                    "The newest short test reports a read failure. Use that line's first-error LBA, which can differ from older tests.".into()
                },
            },
            Check {
                label: "Other disks protected",
                passed: other.is_none(),
                detail: other.map_or_else(
                    || "The other ATA disks retained their original test history.".into(),
                    |(_, (disk, _))| format!("A test was started on {}. Reset the attempt if you need to restore its starting history.", disk.serial),
                ),
            },
        ]
    }

    pub fn monitoring_checks(&self, goal: &Monitoring, saved: &str) -> Vec<Check> {
        let rules = match monitor::parse(saved) {
            Ok(rules) => rules,
            Err(detail) => {
                return vec![Check {
                    label: "Readable configuration",
                    passed: false,
                    detail,
                }];
            }
        };
        let mut checks = Vec::new();
        for (layout, main, auxiliary) in [
            (
                Layout::Current,
                "Primary disk · current layout",
                "Auxiliary disk · current layout",
            ),
            (
                Layout::Maintenance,
                "Primary disk · maintenance layout",
                "Auxiliary disk · maintenance layout",
            ),
        ] {
            let watched = monitor::preview(&rules, &self.inventory, layout);
            let effective = |serial: &str| {
                watched
                    .iter()
                    .filter(|entry| entry.disk.is_some_and(|disk| disk.serial == serial))
                    .collect::<Vec<_>>()
            };
            let target = effective(&goal.primary_serial);
            let aux = effective(&goal.auxiliary_serial);
            let target_ok =
                matches!(target.as_slice(), [row] if row.rule.health && row.rule.selftest);
            let auxiliary_ok =
                matches!(aux.as_slice(), [row] if row.rule.health && !row.rule.selftest);
            let shape_ok = watched.len() == 2 && watched.iter().all(|row| row.disk.is_some());
            checks.push(Check {
                label: main,
                passed: target_ok && shape_ok,
                detail: if target_ok && shape_ok { "One rule watches this disk's health and self-test log.".into() }
                    else if !shape_ok { "The preview must resolve exactly two entries. Look for an unused or extra device name.".into() }
                    else { "Use one rule for the required disk with both -H and -l selftest. Compare the actual resolved serial in the preview.".into() },
            });
            checks.push(Check {
                label: auxiliary,
                passed: auxiliary_ok,
                detail: if auxiliary_ok { "The original auxiliary disk still receives the requested health check.".into() }
                    else { "Keep one health-only rule on the original auxiliary disk. Compare its serial under this layout.".into() },
            });
        }
        checks
    }
}

const TRANSCRIPT_LIMIT: usize = 8;
const REPLY_LIMIT: usize = 8192;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transcript {
    #[serde(deserialize_with = "crate::cron::text::<_,{command::INPUT_LIMIT}>")]
    pub command: String,
    #[serde(deserialize_with = "crate::cron::text::<_,REPLY_LIMIT>")]
    pub output: String,
    pub status: Option<u8>,
}

fn transcript<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Transcript>, D::Error> {
    crate::cron_workspace::sequence::<_, Transcript, TRANSCRIPT_LIMIT>(d)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub state: State,
    pub ticket: Ticket,
    #[serde(deserialize_with = "crate::cron::text::<_,{command::INPUT_LIMIT}>")]
    pub command: String,
    #[serde(deserialize_with = "transcript")]
    pub transcript: Vec<Transcript>,
    #[serde(deserialize_with = "crate::cron::text::<_,{monitor::LIMIT}>")]
    pub monitor_draft: String,
    #[serde(deserialize_with = "crate::cron::text::<_,{monitor::LIMIT}>")]
    pub monitor_saved: String,
    pub checked: bool,
}

impl Work {
    pub fn new(task: &Task) -> Self {
        let config = match &task.goal {
            Assignment::Diagnostic(_) => String::new(),
            Assignment::Monitoring(goal) => goal.initial_config.clone(),
        };
        Self {
            state: State::new(&task.inventory),
            ticket: Ticket::default(),
            command: String::new(),
            transcript: Vec::new(),
            monitor_draft: config.clone(),
            monitor_saved: config,
            checked: false,
        }
    }

    pub fn validate(&self, task: &Task) -> Result<()> {
        self.state.validate(&task.inventory)?;
        if self.command.len() > command::INPUT_LIMIT
            || self.ticket.first_lba.len() > 24
            || self.monitor_draft.len() > monitor::LIMIT
            || self.transcript.len() > TRANSCRIPT_LIMIT
            || self.transcript.iter().any(|line| {
                line.command.len() > command::INPUT_LIMIT || line.output.len() > REPLY_LIMIT
            })
        {
            return Err(
                "A saved SMART command, ticket, configuration or output exceeds its limit.".into(),
            );
        }
        match task.goal {
            Assignment::Diagnostic(_)
                if self.monitor_draft.is_empty() && self.monitor_saved.is_empty() =>
            {
                Ok(())
            }
            Assignment::Monitoring(_)
                if self.state == State::new(&task.inventory)
                    && self.ticket == Ticket::default()
                    && self.transcript.is_empty() =>
            {
                monitor::parse(&self.monitor_saved).map(|_| ())
            }
            _ => Err("The saved SMART state belongs to a different kind of activity.".into()),
        }
    }

    pub fn run(&mut self, task: &Task) {
        let input = std::mem::take(&mut self.command);
        if input.trim().is_empty() {
            return;
        }
        let before = self.state.clone();
        let result =
            command::parse(&input).and_then(|cmd| self.state.execute(&task.inventory, cmd));
        let (output, status) = match result {
            Ok(reply) => (reply.text, Some(reply.status)),
            Err(detail) => (detail, None),
        };
        if self.state != before {
            self.checked = false;
        }
        self.append(Transcript {
            command: input[..input.floor_char_boundary(command::INPUT_LIMIT)].into(),
            output: output[..output.floor_char_boundary(REPLY_LIMIT)].into(),
            status,
        });
    }

    pub fn advance(&mut self, task: &Task) {
        if self
            .state
            .disks
            .iter()
            .any(|disk| disk.remaining_minutes.is_some())
        {
            self.checked = false;
            let events = self.state.advance(&task.inventory);
            let output = if events.is_empty() {
                "Advanced one minute. A supplied self-test is still running.".into()
            } else {
                events.join("\n")
            };
            self.append(Transcript {
                command: "Advance virtual time".into(),
                output,
                status: None,
            });
        }
    }

    pub fn save_monitor(&mut self) -> Result<()> {
        monitor::parse(&self.monitor_draft)?;
        self.monitor_saved.clone_from(&self.monitor_draft);
        self.checked = false;
        Ok(())
    }

    pub fn checks(&self, task: &Task) -> Vec<Check> {
        match &task.goal {
            Assignment::Diagnostic(goal) => task.diagnostic_checks(goal, &self.state, &self.ticket),
            Assignment::Monitoring(goal) => {
                let mut results = task.monitoring_checks(goal, &self.monitor_saved);
                if self.monitor_draft != self.monitor_saved {
                    results.insert(0, Check {
                        label: "Save your edits",
                        passed: false,
                        detail: "The monitoring preview still reads the previously saved file. Save configuration before you check.".into(),
                    });
                }
                results
            }
        }
    }

    fn append(&mut self, line: Transcript) {
        if self.transcript.len() >= TRANSCRIPT_LIMIT {
            self.transcript.remove(0);
        }
        self.transcript.push(line);
    }
}
