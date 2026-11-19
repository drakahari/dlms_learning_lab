//! Source-owned guest tasks; separate, bounded practice and assignment drafts.
use crate::{guest_driver as core, rpm_packages as rpm, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_TASK: usize = 192 * 1024;
pub const MAX_SESSION: usize = 192 * 1024;
pub const MAX_ACTION: usize = 32 * 1024;

fn within<T: Serialize>(v: &T, limit: usize) -> bool {
    struct Count(usize, usize);
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(b.len())
                .filter(|n| *n <= self.1)
                .ok_or_else(|| std::io::Error::other("Guest workspace encoding limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count(0, limit), v).is_ok()
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_".contains(&b))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub facts: core::Facts,
    pub initial: core::Runtime,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,4>")]
    pub devices: Vec<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,4>")]
    pub protected_drivers: Vec<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,rpm::Identity,4>")]
    pub protected_packages: Vec<rpm::Identity>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub detail: String,
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if !within(self, MAX_TASK) {
            return Err("Guest task exceeds its bounded source size.".into());
        }
        self.initial.validate(&self.facts)?;
        if self.devices.is_empty()
            || self.devices.len() > 4
            || self.protected_drivers.len() > 4
            || self.protected_packages.len() > 4
        {
            return Err(
                "Guest tasks need one to four device outcomes and bounded protection requirements."
                    .into(),
            );
        }
        let mut devices = BTreeSet::new();
        for slot in &self.devices {
            let d = self
                .facts
                .modules
                .devices
                .iter()
                .find(|d| d.slot == *slot)
                .ok_or("Target device is not presented by this source.")?;
            if !devices.insert(slot)
                || self.facts.modules.modules[&d.driver].kernel_release
                    != self.facts.modules.kernel_release
            {
                return Err(
                    "Device outcomes need unique slots and supplied current-kernel drivers.".into(),
                );
            }
        }
        let mut protected = BTreeSet::new();
        for n in &self.protected_drivers {
            if !self.initial.modules.loaded.contains_key(n) || !protected.insert(n) {
                return Err(
                    "Protected drivers must be unique and actually loaded in the supplied image."
                        .into(),
                );
            }
        }
        let mut packages = BTreeSet::new();
        for identity in &self.protected_packages {
            if !packages.insert(identity)
                || !self
                    .initial
                    .packages
                    .installed
                    .iter()
                    .any(|p| p.identity == *identity)
            {
                return Err(
                    "Protected package identities must be unique installed source records.".into(),
                );
            }
        }
        Ok(())
    }
    fn bound(&self, r: &core::Runtime) -> Result<(), String> {
        self.validate()?;
        r.validate(&self.facts)?;
        if r.modules.config != self.initial.modules.config {
            return Err("Module options are supplied read-only in this image-support task.".into());
        }
        if r.packages.available != self.initial.packages.available
            || r.packages.cached != self.initial.packages.cached
        {
            return Err(
                "Available local archives and repository facts are supplied by the lesson.".into(),
            );
        }
        Ok(())
    }
    pub fn checks(&self, r: &core::Runtime) -> Vec<Check> {
        if let Err(e) = self.bound(r) {
            return vec![Check {
                passed: false,
                label: "Read this workspace".into(),
                detail: e,
            }];
        }
        let mut rows = vec![];
        for slot in &self.devices {
            let d = self
                .facts
                .modules
                .devices
                .iter()
                .find(|d| d.slot == *slot)
                .unwrap();
            let now = r.modules.loaded.contains_key(&d.driver);
            rows.push(Check{passed:now,label:format!("{}: driver ready now",d.description),detail:if now {format!("{} is the driver in use for {}.",d.driver,d.slot)}else{format!("No active {} driver for {}. Inspect the presented device, its package files and the module command diagnostic; installing a package alone does not activate a driver.",d.driver,d.slot)}});
            let trial = r.fresh_request(&self.facts, &d.driver);
            let future = trial.as_ref().is_ok_and(|o| o.status == 0);
            let why = match trial {
                Ok(o) if o.status != 0 => o.stderr,
                Err(e) => e,
                _ => String::new(),
            };
            rows.push(Check{passed:future,label:format!("{}: fresh driver request",d.description),detail:if future {"The saved files and lookup can resolve this driver and its prerequisites from an unloaded state. This is not a boot or connectivity test.".into()}else{format!("A running instance alone is insufficient. A separate unloaded request fails: {why} Inspect object presence and lookup; restore the matching supplied support, then rebuild lookup if needed.")}});
        }
        let mut closure = BTreeSet::new();
        fn dependencies(n: &str, t: &Task, all: &mut BTreeSet<String>) {
            if all.insert(n.into()) {
                for dep in &t.facts.modules.modules[n].dependencies {
                    dependencies(dep, t, all);
                }
            }
        }
        for n in &self.protected_drivers {
            dependencies(n, self, &mut closure);
        }
        let keep = closure
            .iter()
            .all(|n| r.modules.loaded.get(n) == self.initial.modules.loaded.get(n));
        rows.push(Check{passed:keep,label:"Retain the supplied active providers".into(),detail:if keep {"Protected drivers and their effective prerequisite settings are retained.".into()}else{"A protected driver or its loaded dependency changed. Restore its original effective state; repairing one device must not dismantle another provider.".into()}});
        let packages = self.protected_packages.iter().all(|identity| {
            r.packages
                .installed
                .iter()
                .find(|p| p.identity == *identity)
                == self
                    .initial
                    .packages
                    .installed
                    .iter()
                    .find(|p| p.identity == *identity)
        });
        rows.push(Check{passed:packages,label:"Retain protected package versions".into(),detail:if packages {"The required supplied package records retain their original versions.".into()}else{"Restore the original protected package versions. Reinstalling that same supplied archive is allowed; removing or replacing its record does not meet this handover.".into()}});
        let state = rpm::State::new(self.facts.packages.clone(), r.packages.clone()).unwrap();
        let dependencies = state.dependency_issues().unwrap();
        let missing = dependencies
            .iter()
            .take(3)
            .map(|issue| {
                let constraint = issue
                    .requirement
                    .constraint
                    .as_ref()
                    .map(|c| {
                        let relation = match c.relation {
                            rpm::Relation::Eq => "=",
                            rpm::Relation::Lt => "<",
                            rpm::Relation::Le => "<=",
                            rpm::Relation::Ge => ">=",
                            rpm::Relation::Gt => ">",
                        };
                        format!(" {relation} {}", c.evr.as_str())
                    })
                    .unwrap_or_default();
                format!(
                    "{} needs {}{constraint}",
                    issue.package.name, issue.requirement.capability
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        rows.push(Check{passed:dependencies.is_empty(),label:"Resolve declared package requirements".into(),detail:if dependencies.is_empty(){"No installed package has an unmet declared requirement.".into()}else{format!("{} unmet requirement(s): {missing}. Compare these with the supplied kernel support; an installed record alone does not establish compatibility.",dependencies.len())}});

        rows
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Run {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        input: String,
    },
    Package {
        action: rpm::Action,
    },
    Check,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageOperation {
    Install,
    Replace,
    Reinstall,
    Remove,
}
impl PackageOperation {
    pub fn label(self) -> &'static str {
        match self {
            Self::Install => "Install",
            Self::Replace => "Replace / upgrade",
            Self::Reinstall => "Reinstall",
            Self::Remove => "Remove installed package",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,24>")]
    pub reference: Vec<Action>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,24>")]
    pub example_steps: Vec<Action>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !id(&self.source_id)
            || self.reference.is_empty()
            || self.reference.len() > 24
            || self.example_steps.is_empty()
            || self.example_steps.len() > 24
        {
            return Err(
                "Guest definition needs schema1, a stable identifier and bounded authored methods."
                    .into(),
            );
        }
        if self.assignment.facts == self.example.facts
            && self.assignment.initial == self.example.initial
        {
            return Err("The guided example must have its own supplied values and initial work, separate from the assignment.".into());
        }
        self.assignment.validate()?;
        self.example.validate()
    }
    pub fn validate_reachability(&self) -> Result<(), String> {
        self.validate()?;
        if self
            .assignment
            .checks(&self.assignment.initial)
            .iter()
            .all(|r| r.passed)
        {
            return Err("An independent assignment must need a real repair.".into());
        }
        for (task, steps) in [
            (&self.assignment, &self.reference),
            (&self.example, &self.example_steps),
        ] {
            let mut scope = Scope::new(task);
            for step in steps {
                scope.apply(task, step)?;
            }
            if !task.checks(&scope.runtime).iter().all(|r| r.passed) {
                return Err(
                    "The taught method must reach every stated guest-image outcome.".into(),
                );
            }
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        s.validate()?;
        if s.schema_version != 1 || s.source_id != self.source_id {
            return Err("Saved guest work belongs to a different lesson or unsupported version; retain the original.".into());
        }
        if !within(s, MAX_SESSION) {
            return Err("Guest session exceeds its saved-size bound.".into());
        }
        s.assignment.bound(&self.assignment)?;
        s.practice.bound(&self.example)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        if !s.assignment.checked {
            return vec![(false, "Check the current assignment configuration.".into())];
        }
        self.assignment
            .checks(&s.assignment.runtime)
            .into_iter()
            .map(|r| (r.passed, format!("{}: {}", r.label, r.detail)))
            .collect()
    }
    pub fn current(&self, s: &Session) -> bool {
        self.bound(s).is_ok()
            && s.assignment.checked
            && self
                .assignment
                .checks(&s.assignment.runtime)
                .iter()
                .all(|r| r.passed)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub runtime: core::Runtime,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub output: String,
    pub checked: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub selected_archive: String,
    pub operation: Option<PackageOperation>,
}
impl Scope {
    fn new(t: &Task) -> Self {
        Self {
            runtime: t.initial.clone(),
            input: String::new(),
            output: String::new(),
            checked: false,
            selected_archive: String::new(),
            operation: None,
        }
    }
    pub fn selected_action(&self, task: &Task) -> Result<Action, String> {
        self.bound(task)?;
        let archive = task
            .facts
            .packages
            .archives
            .get(&self.selected_archive)
            .ok_or("Choose a supplied archive first.")?;
        let action = match self
            .operation
            .ok_or("Choose the package action you want to apply.")?
        {
            PackageOperation::Install => rpm::Action::InstallSet {
                archives: vec![self.selected_archive.clone()],
            },
            PackageOperation::Replace => rpm::Action::UpgradeSet {
                archives: vec![self.selected_archive.clone()],
            },
            PackageOperation::Reinstall => rpm::Action::ReinstallExact {
                archive: self.selected_archive.clone(),
            },
            PackageOperation::Remove => rpm::Action::EraseSet {
                packages: vec![archive.identity.clone()],
            },
        };
        Ok(Action::Package { action })
    }
    fn append(&mut self, text: &str) {
        const MARKER: &str = "[Older terminal results omitted]\n";
        let mut joined = if self.output.is_empty() {
            text.to_owned()
        } else {
            format!("{}\n\n{text}", self.output)
        };
        if joined.len() > 32768 {
            let mut start = joined.len() - (32768 - MARKER.len());
            while !joined.is_char_boundary(start) {
                start += 1;
            }
            joined = format!("{MARKER}{}", &joined[start..]);
        }
        self.output = joined;
    }
    pub fn note_package_failure(&mut self, error: &str) {
        let error = crate::rpm_packages_workspace::notice(error);
        self.append(&format!("[Native package action not applied] {error}"));
    }
    /// Native terminal history only: an unsupported request did not run and gets
    /// no invented utility exit status. Runtime and current grading stay intact.
    pub fn note_failure(&mut self, input: &str, error: &str) {
        let input = crate::rpm_packages_workspace::notice(input);
        let error = crate::rpm_packages_workspace::notice(error);
        self.append(&format!("$ {input}\n[Not applied] {error}"));
    }
    fn bound(&self, t: &Task) -> Result<(), String> {
        t.bound(&self.runtime)?;
        if self.input.len() > 4096
            || self.output.len() > 32768
            || (!self.selected_archive.is_empty()
                && !t
                    .initial
                    .packages
                    .available
                    .contains(&self.selected_archive))
            || self.input.contains('\0')
            || self.output.contains('\0')
        {
            return Err(
                "Guest command/output/archive selection exceeds the supplied saved-work bounds."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn apply(&mut self, t: &Task, a: &Action) -> Result<Vec<Check>, String> {
        self.bound(t)?;
        if !within(a, MAX_ACTION) {
            return Err("Guest action exceeds its bounded size.".into());
        }
        let before = self.runtime.clone();
        let output = match a {
            Action::Run { input } => format!(
                "$ {input}\n{}",
                self.runtime.run(&t.facts, input)?.display()
            ),
            Action::Package { action } => {
                let result = self.runtime.package(&t.facts, action)?;
                format!(
                    "Simulated package action applied to {} package(s). Loaded drivers and lookup were not automatically changed.",
                    result.changes.len()
                )
            }
            Action::Check => {
                self.checked = true;
                return Ok(t.checks(&self.runtime));
            }
        };
        // Observations/retained diagnostics are not configuration changes. A real
        // edit always clears current credit, even if it happens to remain valid.
        if self.runtime.packages != before.packages
            || self.runtime.modules.loaded != before.modules.loaded
            || self.runtime.modules.index != before.modules.index
            || self.runtime.modules.config != before.modules.config
        {
            self.checked = false;
        }
        self.append(&output);
        self.bound(t)?;
        Ok(t.checks(&self.runtime))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub schema_version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || !id(&self.source_id) || !within(self, MAX_SESSION) {
            return Err(
                "Unknown guest workspace version, identity or oversized saved work.".into(),
            );
        }
        for scope in [&self.assignment, &self.practice] {
            if scope.input.len() > 4096
                || scope.output.len() > 32768
                || scope.selected_archive.len() > 32
                || scope.input.contains('\0')
                || scope.output.contains('\0')
            {
                return Err(
                    "Guest workspace command, output or selection is outside its bounded form."
                        .into(),
                );
            }
        }
        Ok(())
    }
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            schema_version: 1,
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment),
            practice: Scope::new(&d.example),
        })
    }
    pub fn action(
        &mut self,
        d: &Definition,
        a: &Action,
        practice: bool,
    ) -> Result<Vec<Check>, String> {
        d.bound(self)?;
        let result = if practice {
            self.practice.apply(&d.example, a)
        } else {
            self.assignment.apply(&d.assignment, a)
        };
        d.bound(self)?;
        result
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.bound(self)?;
        if practice {
            self.practice = Scope::new(&d.example)
        } else {
            self.assignment = Scope::new(&d.assignment)
        };
        Ok(())
    }
}
