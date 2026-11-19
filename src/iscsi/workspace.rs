//! Outcome checks and persisted terminal work for the supplied iSCSI exercises.
use super::{Connection, Initiator, Inventory, Key, Result, command};
use serde::{Deserialize, Serialize};

pub const OUTPUT_LIMIT: usize = 8192;
const HISTORY_LIMIT: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Goal {
    pub target: Key,
    pub lun: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,18>")]
    pub wwn: String,
    pub automatic_at_startup: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub inventory: Inventory,
    pub initial: Initiator,
    pub goal: Goal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub label: &'static str,
    pub passed: bool,
    pub feedback: String,
}

impl Task {
    pub fn validate(&self) -> Result<()> {
        self.inventory.validate()?;
        self.initial.validate(&self.inventory)?;
        let export = self
            .inventory
            .export(&self.goal.target)
            .ok_or("The required target/portal pair is absent.")?;
        let disk = self.inventory.device(&self.goal.target, self.goal.lun);
        if !export.login_available || disk.is_none_or(|d| d.wwn != self.goal.wwn) {
            return Err(
                "The required login must be available and expose the supplied LUN and WWN.".into(),
            );
        }
        if self.goal.automatic_at_startup
            && self
                .initial
                .startup_candidates()
                .any(|key| key != &self.goal.target)
        {
            return Err("The initial automatic startup policy would conflict with the protected manual sessions.".into());
        }
        if self.passed(&self.initial) {
            return Err("The supplied initial iSCSI exercise is already solved.".into());
        }
        Ok(())
    }

    pub fn startup_plan<'a>(
        &'a self,
        state: &'a Initiator,
    ) -> impl Iterator<Item = (&'a Key, bool)> {
        state.startup_candidates().map(|key| {
            (
                key,
                self.inventory
                    .export(key)
                    .is_some_and(|e| e.login_available),
            )
        })
    }

    pub fn checks(&self, state: &Initiator) -> Vec<Check> {
        let target = &self.goal.target;
        let saved = state.node(target).is_some();
        let connected = state.session(target).is_some();
        let mut checks = vec![
            Check {
                label: "Required node record",
                passed: saved,
                feedback: if saved {
                    format!("The node database includes {target}.")
                } else {
                    "No record matches both the required target name and portal. Check the SendTargets results.".into()
                },
            },
            Check {
                label: "Active session and disk identity",
                passed: connected
                    && self
                        .inventory
                        .device(target, self.goal.lun)
                        .is_some_and(|disk| disk.wwn == self.goal.wwn),
                feedback: if connected {
                    self.inventory.device(target, self.goal.lun).map_or_else(
                        || format!("The session does not expose LUN {}.", self.goal.lun),
                        |disk| {
                            if disk.wwn == self.goal.wwn {
                                format!(
                                    "The session exposes LUN {} as {} with WWN {}.",
                                    disk.lun, disk.name, disk.wwn
                                )
                            } else {
                                format!(
                                    "LUN {} has WWN {}. Compare it with the inventory ticket.",
                                    disk.lun, disk.wwn
                                )
                            }
                        },
                    )
                } else if saved {
                    "The correct node record is saved. Start a session for that exact record and inspect its attached disks.".into()
                } else {
                    "There is no active session for the required target and portal. A discovery row or a different session cannot expose this disk.".into()
                },
            },
            self.check_other_sessions(state),
        ];
        if self.goal.automatic_at_startup {
            checks.push(self.check_startup(state));
        }
        checks
    }

    fn check_other_sessions(&self, state: &Initiator) -> Check {
        let missing = self
            .initial
            .active
            .iter()
            .filter(|s| s.key != self.goal.target)
            .find(|s| !state.active.contains(s));
        let unexpected = state
            .active
            .iter()
            .find(|s| s.key != self.goal.target && !self.initial.active.contains(s));
        let replacement = |s: &Connection| {
            if state.session(&s.key).is_some() {
                format!(
                    "{} has a replacement session. Reset this attempt if you need to restore the original connection.",
                    s.key
                )
            } else {
                format!(
                    "The original session for {} was disconnected. Reset this attempt to restore it.",
                    s.key
                )
            }
        };
        let feedback = if let Some(session) = missing {
            replacement(session)
        } else if let Some(session) = unexpected {
            format!(
                "There is an extra active session for {}. Log out only that unintended session.",
                session.key
            )
        } else {
            "No required pre-existing session was interrupted, and no unintended session remains."
                .into()
        };
        Check {
            label: "Other sessions",
            passed: missing.is_none() && unexpected.is_none(),
            feedback,
        }
    }

    fn check_startup(&self, state: &Initiator) -> Check {
        let mut plan = self.startup_plan(state);
        let target = &self.goal.target;
        let required = plan.any(|(key, ready)| key == target && ready);
        let extra = state.startup_candidates().find(|key| *key != target);
        let feedback = if let Some(key) = extra {
            format!(
                "{key} is also set to log in automatically. Keep the startup policy manual for other targets."
            )
        } else if required {
            format!("Only {target} is selected for automatic login from a clean start.")
        } else {
            "The required node is not ready for a future automatic login. Check its saved node.startup value. A current session alone does not change it.".into()
        };
        Check {
            label: "Saved startup policy",
            passed: required && extra.is_none(),
            feedback,
        }
    }

    pub fn passed(&self, state: &Initiator) -> bool {
        self.checks(state).iter().all(|result| result.passed)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    #[serde(deserialize_with = "crate::cron::text::<_,{command::INPUT_LIMIT}>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,OUTPUT_LIMIT>")]
    pub output: String,
    pub ok: bool,
}

fn history<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Run>, D::Error> {
    crate::cron_workspace::sequence::<_, Run, HISTORY_LIMIT>(d)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub initiator: Initiator,
    #[serde(deserialize_with = "crate::cron::text::<_,{command::INPUT_LIMIT}>")]
    pub input: String,
    #[serde(deserialize_with = "history")]
    pub history: Vec<Run>,
    pub checked: bool,
}

impl Attempt {
    pub fn new(task: &Task) -> Self {
        Self {
            initiator: task.initial.clone(),
            input: String::new(),
            history: Vec::new(),
            checked: false,
        }
    }

    pub fn validate(&self, task: &Task) -> Result<()> {
        self.initiator.validate(&task.inventory)?;
        if self.input.len() > command::INPUT_LIMIT
            || self.history.len() > HISTORY_LIMIT
            || self
                .history
                .iter()
                .any(|r| r.input.len() > command::INPUT_LIMIT || r.output.len() > OUTPUT_LIMIT)
        {
            return Err("A saved iSCSI command or output exceeds the workspace limit.".into());
        }
        Ok(())
    }

    pub fn run(&mut self, task: &Task) {
        let input = std::mem::take(&mut self.input);
        if input.trim().is_empty() {
            return;
        }
        // The decoder rejects oversized inputs. Keep the same bound on display and storage.
        let before = self.initiator.clone();
        let result =
            command::parse(&input).and_then(|cmd| self.initiator.execute(&task.inventory, cmd));
        let ok = result.is_ok();
        let output = result.unwrap_or_else(|error| error);
        if self.initiator != before {
            self.checked = false;
        }
        if self.history.len() == HISTORY_LIMIT {
            self.history.remove(0);
        }
        self.history.push(Run {
            input: input[..input.floor_char_boundary(command::INPUT_LIMIT)].into(),
            output: output[..output.floor_char_boundary(OUTPUT_LIMIT)].into(),
            ok,
        });
    }
}
