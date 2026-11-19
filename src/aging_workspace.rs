//! Source-bound aging assignments, independent terminal practice and dated checks.
use crate::{accounts::EpochDay, aging as core, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const PREFIX: &str = "AGING PLAN\n";
pub const MAX_SESSION: usize = 256 * 1024;
pub const MAX_DEFINITION: usize = 64 * 1024;
pub const MAX_TRANSCRIPT: usize = 16 * 1024;
type Result<T> = std::result::Result<T, String>;
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn text(s: &str, n: usize) -> bool {
    s.len() <= n
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t' | '\r'))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DateRange {
    pub first: EpochDay,
    pub last: EpochDay,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub day: EpochDay,
    pub expected: core::Observation,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1800>")]
    pub instructions: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub target: String,
    pub initial: core::World,
    pub expected: core::Record,
    pub expiry_range: Option<DateRange>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
fn same_names(a: &core::World, b: &core::World) -> bool {
    a.entries()
        .iter()
        .map(|e| &e.login)
        .eq(b.entries().iter().map(|e| &e.login))
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        if !key(&self.id)
            || self.instructions.trim().is_empty()
            || !text(&self.instructions, 1800)
            || self.initial.entries().len() < 2
            || self.initial.record(&self.target).is_none()
            || !(3..=8).contains(&self.cases.len())
        {
            return Err("Aging task needs a bounded identity, clear task, target plus protected records and three to eight dated samples.".into());
        }
        if let Some(range) = &self.expiry_range
            && (range.first > range.last
                || self
                    .expected
                    .account_expires
                    .is_none_or(|d| d < range.first || d > range.last))
        {
            return Err("The finite account-date range must include the reference date.".into());
        }
        let mut dates = BTreeSet::new();
        for c in &self.cases {
            if c.label.trim().is_empty()
                || !text(&c.label, 120)
                || c.label.chars().any(char::is_control)
                || !dates.insert(c.day)
                || self.expected.observe(c.day) != c.expected
            {
                return Err("Aging samples need unique supplied dates, clear labels and mathematically reachable expected observations.".into());
            }
            if let Some(range) = &self.expiry_range {
                for expiry in [range.first, range.last] {
                    let mut edge = self.expected.clone();
                    edge.account_expires = Some(expiry);
                    if edge.observe(c.day) != c.expected {
                        return Err("Every allowed account date must meet the supplied observations; choose samples outside the allowed interval.".into());
                    }
                }
            }
        }
        if self.checks(&self.initial).iter().all(|r| r.0) {
            return Err("Retain meaningful unfinished aging work in the starter.".into());
        }
        Ok(())
    }
    pub fn compatible(&self, world: &core::World) -> Result<()> {
        if !same_names(&self.initial, world) {
            return Err(
                "Virtual account identities differ from this task; previous state retained.".into(),
            );
        }
        Ok(())
    }
    pub fn observations(&self, world: &core::World) -> Result<Vec<(Case, core::Observation)>> {
        self.compatible(world)?;
        let actual = world
            .record(&self.target)
            .ok_or("The supplied target is missing.")?;
        Ok(self
            .cases
            .iter()
            .map(|c| (c.clone(), actual.observe(c.day)))
            .collect())
    }
    pub fn checks(&self, world: &core::World) -> Vec<(bool, String)> {
        if let Err(e) = self.compatible(world) {
            return vec![(false, e)];
        }
        let actual = world.record(&self.target).unwrap();
        let expected = &self.expected;
        let mut out=vec![
            (actual.last_change==expected.last_change,"Retain the supplied last-change history and any intentional password-reset marker.".into()),
            (actual.minimum==expected.minimum,"Keep the required minimum age; minimum age restricts a change attempt, not ordinary use.".into()),
            (actual.maximum==expected.maximum,"Use the required maximum password age; changing history or disabling expiry is not the requested repair.".into()),
            (actual.warning==expected.warning,"Use the requested warning window before password expiry.".into()),
            (actual.inactive==expected.inactive,"Use the requested inactivity grace, counted after maximum password age.".into()),
            (match &self.expiry_range {Some(r)=>actual.account_expires.is_some_and(|d| d>=r.first&&d<=r.last),None=>actual.account_expires==expected.account_expires},
                "Keep the required finite account-expiry policy; account expiry is separate from password age.".into()),
            (self.initial.entries().iter().filter(|e|e.login!=self.target).all(|e|world.record(&e.login)==Some(&e.record)),
                "Leave every other supplied account's aging record unchanged.".into()),
        ];
        for (case, actual) in self.observations(world).unwrap() {
            let warning = |n: Option<u32>| {
                n.map_or_else(
                    || "no warning".into(),
                    |n| format!("warning: {n} days left"),
                )
            };
            let passed = actual == case.expected;
            out.push((passed,format!("{} · {}: expected {} / {}; actual {} / {}.{}{}",
                case.day.iso(),case.label,case.expected.verdict.label(),warning(case.expected.warning_days),
                actual.verdict.label(),warning(actual.warning_days),
                if actual.change_too_soon!=case.expected.change_too_soon{" The minimum-change-age result differs."}else{""},
                if actual.future_last_change!=case.expected.future_last_change{" The recorded change is unexpectedly in the future; do not invent a password change."}else{""})));
        }
        out
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command(#[serde(deserialize_with = "crate::cron::text::<_,1024>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Command,8>")]
    pub example_worked: Vec<Command>,
}
impl Definition {
    pub fn from_json(raw: &str) -> Result<Self> {
        if raw.len() > MAX_DEFINITION {
            return Err("Aging task exceeds64KiB.".into());
        }
        let d: Self = serde_json::from_str(raw).map_err(|e| format!("Invalid aging task: {e}"))?;
        d.validate()?;
        Ok(d)
    }
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.target == self.example.target
            || self
                .initial
                .initial
                .entries()
                .iter()
                .any(|e| self.example.initial.record(&e.login).is_some())
            || self.example_worked.is_empty()
            || self.example_worked.len() > 8
        {
            return Err("Practice needs separate account identities, starter work and a bounded worked approach.".into());
        }
        let mut world = self.example.initial.clone();
        for command in &self.example_worked {
            world.execute(&command.0)?;
        }
        if !self.example.checks(&world).iter().all(|r| r.0) {
            return Err("The worked practice must satisfy all fields, protected records and dated outcomes.".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        self.validate()?;
        s.validate()?;
        if s.origin != self.initial.id {
            return Err("This saved aging work belongs to another task.".into());
        }
        self.initial.compatible(&s.assignment.world)?;
        self.example.compatible(&s.practice.world)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        if !s.assignment.checked {
            return vec![(false,"Not checked · choose Check aging policy to compare the current records and supplied dates.".into())];
        }
        self.initial.checks(&s.assignment.world)
    }
    pub fn solved(&self, s: &Session) -> bool {
        let rows = self.checks(s);
        !rows.is_empty() && rows.iter().all(|r| r.0)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogLine {
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub text: String,
    pub error: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub world: core::World,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,LogLine,16>")]
    pub transcript: Vec<LogLine>,
    pub checked: bool,
}
impl Scope {
    fn new(t: &Task) -> Self {
        Self {
            world: t.initial.clone(),
            input: String::new(),
            transcript: vec![],
            checked: false,
        }
    }
    pub fn edit_input(&mut self, text: &str) -> Result<()> {
        if text.len() > core::MAX_COMMAND || text.chars().any(|c| c.is_control() && c != '\t') {
            return Err("Use one command of at most1024bytes; previous draft retained.".into());
        }
        self.input = text.into();
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        if self.input.len() > core::MAX_COMMAND
            || self.input.chars().any(|c| c.is_control() && c != '\t')
            || self.transcript.len() > 16
            || self.transcript.iter().any(|l| !text(&l.text, 4096))
            || self.transcript.iter().map(|l| l.text.len()).sum::<usize>() > MAX_TRANSCRIPT
        {
            return Err("Aging input or terminal history exceeds its safe bounds.".into());
        }
        Ok(())
    }
    pub fn execute(&mut self, t: &Task) -> Result<String> {
        self.validate()?;
        t.compatible(&self.world)?;
        let mut next = self.world.clone();
        let result = next.execute(&self.input);
        let result_text = match &result {
            Ok(s) if s.is_empty() => "".into(),
            Ok(s) => s.clone(),
            Err(e) => format!("Error: {e}\n"),
        };
        let line = format!("$ {}\n{result_text}", self.input);
        // The bounded parser and one-record query fit here; reject before logging
        // rather than silently truncating a diagnostic or changing the grade.
        if line.len() > 4096 {
            return Err("Terminal row exceeded its output bound.".into());
        }
        self.world = next;
        self.checked = false;
        self.transcript.push(LogLine {
            text: line,
            error: result.is_err(),
        });
        while self.transcript.len() > 16
            || self.transcript.iter().map(|l| l.text.len()).sum::<usize>() > MAX_TRANSCRIPT
        {
            self.transcript.remove(0);
        }
        result
    }
    pub fn check(&mut self, t: &Task) -> Result<Vec<(bool, String)>> {
        self.validate()?;
        t.compatible(&self.world)?;
        self.checked = true;
        Ok(t.checks(&self.world))
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
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        })
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || !key(&self.origin) {
            return Err("Unknown aging-progress version or identity.".into());
        }
        self.assignment.validate()?;
        self.practice.validate()
    }
    pub fn from_json(raw: &str, d: &Definition) -> Result<Self> {
        if raw.len() > MAX_SESSION {
            return Err("Aging progress exceeds256KiB.".into());
        }
        let s: Self =
            serde_json::from_str(raw).map_err(|e| format!("Invalid aging progress: {e}"))?;
        d.bound(&s)?;
        Ok(s)
    }
    pub fn fill_example(&mut self, d: &Definition) -> Result<()> {
        d.bound(self)?;
        let mut next = self.practice.clone();
        for c in &d.example_worked {
            next.edit_input(&c.0)?;
            next.execute(&d.example)?;
        }
        next.checked = false;
        self.practice = next;
        Ok(())
    }
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String> {
        d.bound(self)?;
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the virtual terminal; this data-only adapter is not a shell command.")?;
        if raw.len() > 16 * 1024 {
            return Err("Reference command plan exceeds its bound.".into());
        }
        #[derive(Deserialize)]
        struct Plan(
            #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Command,8>")]
            Vec<Command>,
        );
        let Plan(commands) =
            serde_json::from_str(raw).map_err(|e| format!("Invalid aging command plan: {e}"))?;
        if commands.is_empty() {
            return Err("Reference plan is empty.".into());
        }
        let mut next = self.assignment.clone();
        for c in commands {
            next.edit_input(&c.0)?;
            next.execute(&d.initial)?;
        }
        let rows = next.check(&d.initial)?;
        self.assignment = next;
        Ok(format!(
            "Aging policy checked: {}/{} outcomes match",
            rows.iter().filter(|r| r.0).count(),
            rows.len()
        ))
    }
}

pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (crate::engine::State::Aging { simulation }, crate::engine::Goal::Aging { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Aging source/state mismatch.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err("Aging source identity, task and supplied records must match.".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::Aging { simulation } = &a.state else {
        return Err("Aging work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible aging attempt metadata; original retained.".into());
    }
    d.bound(simulation)
}
