//! Source-bound time-zone outcomes; no host clock, files or serialized world.
use crate::boot_entry::{Error, ErrorKind};
use crate::timezones::{
    self, Environment, Zone,
    command::{Output, World},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_HISTORY: usize = 96;
pub const CHECK_FORMAT: &str = "%F %T %z %Z";
type Result<T> = std::result::Result<T, String>;
mod bindings;
mod session;
pub use bindings::{HELP, context, relevant, source};
pub use session::{Definition, Draft, Session};
#[cfg(test)]
mod tests;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub text: String,
}
impl Command {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
    fn validate(&self) -> Result<()> {
        if self.text.trim().is_empty() || self.text.len() > 1024 || self.text.contains('\0') {
            Err("Use one nonempty supported command within1,024bytes.".into())
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Moment {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub label: String,
    pub epoch: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Moment,6>")]
    pub moments: Vec<Moment>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub configured: String,
    pub environment: Environment,
    pub clock: i64,
}
impl Snapshot {
    pub fn world(&self) -> Result<World> {
        World::new(
            self.configured.clone(),
            self.environment.clone(),
            self.clock,
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Scope {
    Current {},
    Login {},
    Handover { system_zone: Zone },
}
impl Scope {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Current {} => "Current process",
            Self::Login {} => "Separate login",
            Self::Handover { .. } => "After the supplied zone handover",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Case {
    Configured {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        id: String,
        expected: Zone,
    },
    Date {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        id: String,
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        moment: String,
        scope: Scope,
        #[serde(deserialize_with = "crate::cron::text::<_,128>")]
        expected: String,
    },
}
impl Case {
    fn id(&self) -> &str {
        match self {
            Self::Configured { id, .. } | Self::Date { id, .. } => id,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub label: String,
    pub correct: bool,
    pub feedback: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub facts: Facts,
    pub initial: Snapshot,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Command,48>")]
    pub reference: Vec<Command>,
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        if !identifier(&self.id)
            || (self.facts.moments.len() < 2 || self.facts.moments.len() > 6)
            || !(3..=8).contains(&self.cases.len())
            || self.reference.is_empty()
            || self.reference.len() > 48
        {
            return Err("Use a stable task ID, two to six supplied instants, three to eight outcomes and a bounded reference method.".into());
        }
        let mut moments = BTreeSet::new();
        let mut instants = BTreeSet::new();
        for m in &self.facts.moments {
            if !identifier(&m.id)
                || m.label.trim().is_empty()
                || m.label.len() > 96
                || m.label.chars().any(char::is_control)
                || !moments.insert(&m.id)
                || !instants.insert(m.epoch)
            {
                return Err(
                    "Supplied timestamp IDs, labels and UTC instants must be distinct and bounded."
                        .into(),
                );
            }
            timezones::require_epoch(m.epoch)?;
        }
        let initial = self.initial.world()?;
        if self.initial.clock != self.facts.moments[0].epoch {
            return Err("The virtual now must be the first supplied UTC instant.".into());
        }
        let mut ids = BTreeSet::new();
        let mut used = BTreeSet::new();
        let mut configured = 0;
        let mut current = false;
        let mut other = false;
        for case in &self.cases {
            if !identifier(case.id()) || !ids.insert(case.id()) {
                return Err("Outcome IDs must be unique bounded identifiers.".into());
            }
            match case {
                Case::Configured { .. } => configured += 1,
                Case::Date {
                    moment,
                    scope,
                    expected,
                    ..
                } => {
                    if !moments.contains(moment)
                        || expected.is_empty()
                        || expected.len() > 128
                        || !expected.is_ascii()
                        || expected.contains('\0')
                    {
                        return Err("Date outcomes need a declared instant and bounded C-format expected output.".into());
                    }
                    used.insert(moment);
                    match scope {
                        Scope::Current {} => current = true,
                        _ => other = true,
                    }
                }
            }
        }
        if configured != 1 || used != moments || !current || !other {
            return Err("Check one configured default and every supplied instant across current and independent login/handover contexts.".into());
        }
        if self.checks(&initial).iter().all(|r| r.correct) {
            return Err("Independent work must start with a meaningful incomplete outcome.".into());
        }
        for c in &self.reference {
            c.validate()?;
        }
        let reached = self.replay(&self.reference)?;
        if self.checks(&reached).iter().any(|r| !r.correct) {
            return Err(
                "The supported reference method must reach every actual time-zone outcome.".into(),
            );
        }
        Ok(())
    }
    pub fn replay(&self, history: &[Command]) -> Result<World> {
        if history.len() > MAX_HISTORY {
            return Err("Saved time-zone history exceeds96actions.".into());
        }
        let mut world = self.initial.world()?;
        for action in history {
            action.validate()?;
            world
                .command(&action.text)
                .map_err(|e| format!("Saved command cannot replay in this activity: {e}"))?;
        }
        Ok(world)
    }
    pub fn checks(&self, world: &World) -> Vec<Check> {
        let unchanged = world.clock() == self.initial.clock;
        let mut rows = vec![Check {
            label: "Supplied UTC instant".into(),
            correct: unchanged,
            feedback: if unchanged {
                "The underlying instant is unchanged; offsets change its display."
            } else {
                "This work no longer uses the supplied reference instant."
            }
            .into(),
            expected: Some(self.initial.clock.to_string()),
            actual: Some(world.clock().to_string()),
        }];
        for case in &self.cases {
            match case {
                Case::Configured { expected, .. } => {
                    let correct = Zone::parse(world.configured_name()) == Ok(*expected);
                    rows.push(Check{label:"Configured system zone".into(),correct,
                        feedback:if correct{"The saved default uses the requested regional policy.".into()}else{format!("The configured default is still {}. A date -u view or a process override does not save the requested system zone.",world.configured_name())},
                        expected:Some(expected.name().into()),actual:Some(world.configured_name().into())});
                }
                Case::Date {
                    moment,
                    scope,
                    expected,
                    ..
                } => {
                    let Some(m) = self.facts.moments.iter().find(|m| m.id == *moment) else {
                        rows.push(Check {
                            label: "Missing supplied instant".into(),
                            correct: false,
                            feedback: "Cannot check an undeclared timestamp.".into(),
                            expected: None,
                            actual: None,
                        });
                        continue;
                    };
                    let (name, env) = match scope {
                        Scope::Current {} => (world.configured_name(), world.environment().clone()),
                        Scope::Login {} => (world.configured_name(), Environment { tz: None }),
                        Scope::Handover { system_zone } => {
                            (system_zone.name(), world.environment().clone())
                        }
                    };
                    let actual = World::new(name.into(), env, m.epoch)
                        .and_then(|w| w.at(m.epoch, false, CHECK_FORMAT));
                    let correct = actual.as_ref() == Ok(expected);
                    let feedback = if correct {
                        "This timestamp follows the required policy, including its numeric offset."
                            .into()
                    } else if matches!(scope, Scope::Login {}) {
                        "A separate login reads the configured system zone. Fixing only a command's display or current-process override does not save that default.".into()
                    } else if let Some(value) = world.environment().tz.as_deref() {
                        let explanation = if value.is_empty() {
                            "an empty TZ value (which means UTC)".into()
                        } else {
                            format!("TZ={value}")
                        };
                        format!(
                            "This process still uses {explanation}. Compare that override with the required policy; a correct system default alone cannot remove it."
                        )
                    } else {
                        "This timestamp does not follow the requested regional policy. Check the configured zone; do not compensate by changing the underlying instant.".into()
                    };
                    rows.push(Check {
                        label: format!("{} · {}", scope.label(), m.label),
                        correct,
                        feedback,
                        expected: Some(expected.clone()),
                        actual: Some(actual.unwrap_or_else(|e| format!("Cannot display: {e}"))),
                    });
                }
            }
        }
        rows
    }
}
