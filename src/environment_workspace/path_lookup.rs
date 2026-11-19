//! Optional inherited-worker lookup observations; resolved utilities never run.
use super::{Case, CaseRun, Task};
use crate::{
    filesystem::{Access, path_lookup as core},
    shell::EnvironmentSnapshot,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};

fn rows<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Expected>, D::Error> {
    crate::cron_workspace::sequence::<D, Expected, 8>(d)
}
struct Text<const N: usize>(String);
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        crate::cron::text::<D, N>(d).map(Self)
    }
}
fn paths<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Vec<String>>, D::Error> {
    struct Paths;
    impl<'de> serde::de::Visitor<'de> for Paths {
        type Value = BTreeMap<String, Vec<String>>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("up to4 names with up to4 approved source paths each")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut a: A,
        ) -> Result<Self::Value, A::Error> {
            struct Choices(Vec<String>);
            impl<'de> Deserialize<'de> for Choices {
                fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                    crate::cron_workspace::sequence::<D, Text<1024>, 4>(d)
                        .map(|v| Self(v.into_iter().map(|s| s.0).collect()))
                }
            }
            let mut out = BTreeMap::new();
            while let Some(Text(name)) = a.next_key::<Text<48>>()? {
                if out.len() >= 4 || out.contains_key(&name) {
                    return Err(serde::de::Error::custom("Duplicate/excess PATH query name"));
                }
                let Choices(value) = a.next_value()?;
                out.insert(name, value);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Paths)
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expected {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub case_name: String,
    #[serde(deserialize_with = "paths")]
    pub selected: BTreeMap<String, Vec<String>>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub profile: core::Profile,
    #[serde(deserialize_with = "rows")]
    pub cases: Vec<Expected>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Run {
    Observed {
        observation: core::Observation,
    },
    Error {
        category: ErrorKind,
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        message: String,
    },
}
impl Plan {
    pub fn validate(&self, cases: &[Case]) -> Result<(), String> {
        self.profile.validate().map_err(|e| e.to_string())?;
        if self.cases.len() != cases.len() || !(2..=8).contains(&self.cases.len()) {
            return Err("PATH expectations must match all supplied case names".into());
        }
        let names: BTreeSet<_> = self.profile.names.iter().collect();
        let mut seen = BTreeSet::new();
        for expected in &self.cases {
            if expected.case_name.is_empty()
                || expected.case_name.len() > 120
                || !seen.insert(&expected.case_name)
            {
                return Err("Distinct bounded PATH case names required".into());
            }
            let case = cases
                .iter()
                .find(|c| c.input.name == expected.case_name)
                .ok_or("Unknown PATH expectation case name")?;
            if expected.selected.keys().collect::<BTreeSet<_>>() != names {
                return Err("Every fixed source query needs approved paths".into());
            }
            for (name, choices) in &expected.selected {
                let mut seen = BTreeSet::new();
                if choices.is_empty()
                    || choices.len() > 4
                    || choices.iter().any(|p| {
                        p.len() > 1024
                            || !seen.insert(p)
                            || p.rsplit('/').next() != Some(name.as_str())
                            || !self.profile.tree.nodes.get(p).is_some_and(|n| !n.directory)
                            || !self.profile.trusted_directories.iter().any(|d| {
                                p.rsplit_once('/').is_some_and(|(parent, _)| {
                                    if parent.is_empty() {
                                        d == "/"
                                    } else {
                                        d == parent
                                    }
                                })
                            })
                            || !self.profile.tree.access(
                                &self.profile.tree.user,
                                p,
                                Access::ExecuteFile,
                            )
                    })
                {
                    return Err("Approved PATH selections must be distinct source executable artifact paths".into());
                }
            }
            let path = case
                .parent
                .get("PATH")
                .ok_or("PATH mode needs an exported defined reference binding")?;
            if !path.exported
                || path.value.is_none()
                || case.child.env.get("PATH") != path.value.as_ref()
            {
                return Err("Canonical child PATH must match its exported parent reference".into());
            }
            let observation =
                core::resolve(&self.profile, path.value.as_deref()).map_err(|e| e.to_string())?;
            self.check_observation(&expected.case_name, &observation)?;
        }
        Ok(())
    }
    /// ACTUAL completed child snapshot only; never reads canonical case.env.
    pub fn observe(&self, snapshot: &EnvironmentSnapshot) -> Run {
        match core::resolve(
            &self.profile,
            snapshot.child.get("PATH").map(String::as_str),
        ) {
            Ok(observation) => Run::Observed { observation },
            Err(e) => Run::Error {
                category: match e.kind {
                    core::ErrorKind::Invalid => ErrorKind::Invalid,
                    core::ErrorKind::Unsupported => ErrorKind::Unsupported,
                    core::ErrorKind::Limit => ErrorKind::Limit,
                },
                message: e.message,
            },
        }
    }
    pub fn check(&self, case_name: &str, run: &Run) -> Result<(), String> {
        match run {
            Run::Observed { observation } => self.check_observation(case_name, observation),
            Run::Error { category, message } => {
                Err(format!("Worker lookup {category:?}: {message}"))
            }
        }
    }
    fn check_observation(&self, name: &str, o: &core::Observation) -> Result<(), String> {
        if !o.matches(&self.profile) {
            return Err(
                "Worker lookup observation differs from its readonly source profile".into(),
            );
        }
        let expected = self
            .cases
            .iter()
            .find(|c| c.case_name == name)
            .ok_or("Missing source PATH case expectation")?;
        if let core::Policy::Untrusted { directories } = &o.policy {
            return Err(format!(
                "Worker PATH contains directories outside its readonly trusted list: {}. Check the entire path, including unused suffixes.",
                directories.join(", ")
            ));
        }
        for report in &o.reports {
            match &report.decision {
                core::Decision::Selected { path }
                    if expected
                        .selected
                        .get(&report.name)
                        .is_some_and(|p| p.contains(path)) => {}
                core::Decision::Selected { path } => {
                    return Err(format!(
                        "{} resolves to {path}, not an approved provider for this sample. Check earlier PATH entries.",
                        report.name
                    ));
                }
                core::Decision::NotFound => {
                    return Err(format!(
                        "{} was not found in the worker's PATH. Keep its required supplied directories.",
                        report.name
                    ));
                }
                core::Decision::Denied => {
                    return Err(format!(
                        "{} has no usable executable candidate; inspect execute permission and parent directory search facts.",
                        report.name
                    ));
                }
            }
        }
        Ok(())
    }
}
/// Metadata consistency only; runtime Record proof still decides current credit.
pub(super) fn metadata_valid(task: &Task, row: &CaseRun) -> bool {
    match (&task.path_lookup, &row.path_lookup, &row.bindings) {
        (None, None, _) => true,
        (Some(plan), Some(run), Some(snapshot)) => plan.observe(snapshot) == *run,
        (Some(_), None, None) => row.parent.error.is_some() || row.diagnostic.is_some(),
        _ => false,
    }
}
/// Reserve before lookup so Stop/limits cannot publish an uncharged result.
pub(super) const MAX_CHARGE: usize = core::MAX_PATH + core::MAX_STEPS as usize + 1;
pub(super) fn charge(snapshot: &EnvironmentSnapshot, run: &Run) -> usize {
    let input = snapshot
        .child
        .get("PATH")
        .map_or(0, String::len)
        .min(core::MAX_PATH);
    1 + input
        + match run {
            Run::Observed { observation } => observation.steps as usize,
            Run::Error { .. } => core::MAX_STEPS as usize,
        }
}

/// New opt-in standalone Task calls get the same source budget as a bundled
/// Definition; streaming counts bytes without retaining another serialized copy.
pub(super) fn task_fits(task: &Task) -> bool {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if b.len() > (super::MAX_SESSION / 4).saturating_sub(self.0) {
                return Err(std::io::Error::other("PATH task byte limit"));
            }
            self.0 += b.len();
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(&mut Counter(0), task).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stop_after_actual_lookup_before_child_does_not_publish_a_record() {
        let task: Task = serde_json::from_str(include_str!(
            "../../tests/fixtures/environment-path-lookup.json"
        ))
        .unwrap();
        let mut job =
            super::super::Job::new(&task, "export PATH=\"$TOOL_DIR:$BASE_PATH\"\n").unwrap();
        for _ in 0..512 {
            job.advance(1);
            if job.path_lookup.is_some() {
                break;
            }
        }
        assert!(
            matches!(job.path_lookup, Some(Run::Observed { .. })),
            "must reach actual snapshot-derived lookup, not assume a frame count"
        );
        assert!(job.parent.is_some());
        assert!(job.vm.is_none());
        assert!(!job.done());
        job.stop();
        assert!(job.record().is_none());
    }
}
