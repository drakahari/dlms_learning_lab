//! Bounded editor/replay lessons over the pure selector engine.
use crate::{linux_logging as core, pack::Scenario, progress::Attempt, script_teaching::View};
use serde::{Deserialize, Serialize};
pub const PREFIX: &str = "SYSLOG CONFIG\n";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub task: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub starter: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,core::Event,12>")]
    pub events: Vec<core::Event>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,core::Policy,12>")]
    pub policy: Vec<core::Policy>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,4>")]
    pub destinations: Vec<String>,
    pub initial: Case,
    pub example: Case,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub example_worked: String,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.destinations.is_empty()
            || self.destinations.len() > 4
            || self
                .destinations
                .iter()
                .any(|s| !core::valid_destination(s))
            || self
                .destinations
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.destinations.len()
            || self.initial == self.example
        {
            return Err(
                "Syslog task needs distinct cases and unique bounded virtual destinations".into(),
            );
        }
        for case in [&self.initial, &self.example] {
            if case.task.is_empty()
                || case.task.len() > 2048
                || case.starter.len() > 8192
                || case.events.is_empty()
                || case.events.len() > 12
                || case.policy.is_empty()
                || case.policy.len() > 12
            {
                return Err("Syslog task exceeds its bounds or has no sample events/policy".into());
            }
            core::parse(&case.starter, &self.destinations)?;
            for event in &case.events {
                event.validate()?;
            }
            for p in &case.policy {
                if !self.destinations.contains(&p.destination)
                    || p.facilities.is_empty()
                    || p.facilities.len() > 24
                    || p.facilities.iter().any(|n| *n >= 24)
                    || p.severities.is_empty()
                    || p.severities.len() > 8
                    || p.severities.iter().any(|n| *n >= 8)
                    || p.copies != 1
                    || p.facilities
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        != p.facilities.len()
                    || p.severities
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        != p.severities.len()
                {
                    return Err("Invalid syslog outcome policy".into());
                }
            }
        }
        let example = core::parse(&self.example_worked, &self.destinations)?;
        if !core::compare(&example, &self.example.policy)
            .iter()
            .all(|r| r.0)
        {
            return Err("Worked syslog configuration does not satisfy the example policy".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        s.validate()?;
        for scope in [&s.assignment, &s.practice] {
            if let Some(config) = &scope.last_run {
                core::parse(config, &self.destinations)?;
            }
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        let (scope, case) = if practice {
            (&s.practice, &self.example)
        } else {
            (&s.assignment, &self.initial)
        };
        if scope.last_run.as_deref() != Some(scope.draft.as_str()) {
            return vec![(false,"Not checked for this draft. Select Run routing test to parse the edited configuration and replay the supplied events.".into())];
        }
        match core::parse(&scope.draft, &self.destinations) {
            Ok(rules) => core::compare(&rules, &case.policy),
            Err(e) => vec![(false, e)],
        }
    }
}
fn optional_text<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> Result<Option<String>, D::Error> {
    struct Optional<const N: usize>;
    impl<'de, const N: usize> serde::de::Visitor<'de> for Optional<N> {
        type Value = Option<String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "null or bounded text up to {N} bytes")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            crate::cron::text::<D, N>(d).map(Some)
        }
    }
    d.deserialize_option(Optional::<N>)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub draft: String,
    #[serde(deserialize_with = "optional_text::<_,8192>")]
    pub last_run: Option<String>,
    #[serde(deserialize_with = "optional_text::<_,1024>")]
    pub diagnostic: Option<String>,
}
impl Scope {
    fn new(case: &Case) -> Self {
        Self {
            draft: case.starter.clone(),
            last_run: None,
            diagnostic: None,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 40 * 1024
        {
            return Err("Syslog workspace version or40KiB bound exceeded".into());
        }
        for scope in [&self.assignment, &self.practice] {
            if scope.draft.len() > 8192
                || scope.last_run.as_ref().is_some_and(|s| s.len() > 8192)
                || scope.diagnostic.as_ref().is_some_and(|s| s.len() > 1024)
            {
                return Err("Syslog editor/output limit exceeded".into());
            }
        }
        Ok(())
    }
    pub fn execute_reference(&mut self, d: &Definition, text: &str) -> Result<String, String> {
        let raw = text
            .strip_prefix(PREFIX)
            .ok_or("Use the simulated selector editor; this is not a Linux command terminal")?;
        if raw.len() > 8192 {
            return Err("Selector text exceeds8192 bytes".into());
        }
        d.bound(self)?;
        let mut next = self.clone();
        next.assignment.draft = raw.into();
        next.run(d, false)?;
        d.bound(&next)?;
        *self = next;
        Ok("Virtual selector fragment tested".into())
    }
    pub fn run(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.bound(self)?;
        let scope = if practice {
            &mut self.practice
        } else {
            &mut self.assignment
        };
        match core::parse(&scope.draft, &d.destinations) {
            Ok(_) => {
                scope.last_run = Some(scope.draft.clone());
                scope.diagnostic = None;
                Ok(())
            }
            Err(e) => {
                scope.last_run = None;
                scope.diagnostic = Some(e.clone());
                Err(e)
            }
        }
    }
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::LinuxLogging { simulation },
        crate::engine::Goal::LinuxLogging { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Syslog task/state mismatch".into());
    };
    definition.validate()?;
    if definition.initial.task != s.objective {
        return Err("Syslog task must match its independent requirements".into());
    }
    if **simulation != Session::new(definition) {
        return Err("Syslog initial draft does not match the independent authored cases".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::LinuxLogging { simulation } = &a.state else {
        return Err("Saved syslog task belongs to another workspace".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible syslog attempt metadata; original retained".into());
    }
    d.bound(simulation)
}
