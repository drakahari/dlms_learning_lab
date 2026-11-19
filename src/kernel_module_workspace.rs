//! Source-held module tasks and independent practice. Saved work is never authority.
use crate::{kernel_modules as core, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const CHECK_ACTION: &str = "@module-check";
pub const SAVE_PREFIX: &str = "@module-save ";
pub const MAX_SESSION: usize = 256 * 1024;
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
}
fn bounded<T: Serialize>(value: &T, cap: usize) -> bool {
    struct Count(usize, usize);
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(b.len())
                .filter(|n| *n <= self.1)
                .ok_or_else(|| std::io::Error::other("Module workspace encoding limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count(0, cap), value).is_ok()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Assignment,
    Practice,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub module: String,
    pub parameters: BTreeMap<String, i64>,
    pub saved_options: bool,
    pub current_index: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub request_names: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Activity {
    pub catalogue: core::Catalogue,
    pub initial: core::Runtime,
    pub target: Target,
}
impl Activity {
    pub fn named(&self) -> bool {
        !self.target.request_names.is_empty()
    }
    pub fn step_label(&self, step: &Step) -> String {
        if self.named() && matches!(step, Step::SaveConfig { .. }) {
            "Edit and save the practice name bindings".into()
        } else {
            step.label()
        }
    }
    pub fn command_help(&self) -> &'static str {
        if self.named() {
            core::NAMED_HELP
        } else {
            core::HELP
        }
    }
    fn execute(&self, r: &mut core::Runtime, input: &str) -> Result<core::Outcome, String> {
        if self.named() {
            r.execute_named(&self.catalogue, input)
        } else {
            r.execute(&self.catalogue, input)
        }
    }
    fn closure(&self) -> BTreeSet<String> {
        fn add(n: &str, c: &core::Catalogue, set: &mut BTreeSet<String>) {
            if !set.insert(n.to_owned()) {
                return;
            }
            for d in &c.modules[n].dependencies {
                add(d, c, set);
            }
        }
        let mut set = BTreeSet::new();
        add(&self.target.module, &self.catalogue, &mut set);
        set
    }
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate(&self.catalogue)?;
        let m = self
            .catalogue
            .modules
            .get(&self.target.module)
            .ok_or("Target module must be supplied.")?;
        if m.kernel_release != self.catalogue.kernel_release
            || self.target.parameters.len() != m.parameters.len()
            || self.target.parameters.iter().any(|(n, v)| {
                m.parameters
                    .get(n)
                    .is_none_or(|p| !(p.minimum..=p.maximum).contains(v))
            })
        {
            return Err(
                "Target needs the current release and every declared valid effective parameter."
                    .into(),
            );
        }
        // Invalid integer values may be an intentional starting fault; syntax/other
        // directives are not part of these authored tasks.
        if self.named() {
            if self.target.saved_options || self.target.current_index {
                return Err("Exact-name tasks preserve physical options and grade their declared requests; combined option/full-index goals are unsupported in this task family.".into());
            }
            if self.target.request_names.len() > 4 {
                return Err("At most four independent named requests are supported.".into());
            }
            let mut names = BTreeSet::new();
            for n in &self.target.request_names {
                let n = core::normalized_name(n)?;
                if self.catalogue.modules.contains_key(&n) || !names.insert(n) {
                    return Err("Named-request outcomes need unique nonphysical names; hyphen/underscore spellings are the same name.".into());
                }
            }
            core::parse_named_config(&self.initial.config, &self.catalogue)?;
        } else {
            core::parse_config(&self.initial.config, &self.catalogue)?;
        }
        Ok(())
    }
    pub fn bound(&self, r: &core::Runtime) -> Result<(), String> {
        r.validate(&self.catalogue)
    }
    pub fn checks(&self, r: &core::Runtime) -> Vec<(bool, String)> {
        if let Err(e) = self.validate().and_then(|_| self.bound(r)) {
            return vec![(false, e)];
        }
        if self.named() {
            return self.named_checks(r);
        }
        let n = &self.target.module;
        let loaded = r.loaded.get(n);
        let current = loaded.is_some_and(|l| l.parameters == self.target.parameters);
        let values = |p: &BTreeMap<String, i64>| {
            if p.is_empty() {
                "no parameters".to_owned()
            } else {
                p.iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        };
        let effective = match loaded {
            None => format!(
                "{n} is not loaded. Inspect its supplied object/dependencies and the command diagnostic; naming a possible driver is not activating it."
            ),
            Some(l) if !current => format!(
                "{n} is loaded with {}; the task needs {}. Saving options or repeating modprobe does not update an existing instance; inspect its effective parameter and reload safely.",
                values(&l.parameters),
                values(&self.target.parameters)
            ),
            Some(_) => format!("{n} is loaded with the requested effective parameters."),
        };
        let allowed = self.closure();
        let protected = r
            .loaded
            .iter()
            .filter(|(n, _)| !allowed.contains(*n))
            .eq(self
                .initial
                .loaded
                .iter()
                .filter(|(n, _)| !allowed.contains(*n)))
            && allowed
                .iter()
                .filter(|n| n.as_str() != self.target.module)
                .all(|n| {
                    r.loaded.get(n).is_none_or(|l| {
                        let expected = self
                            .initial
                            .loaded
                            .get(n)
                            .map(|p| p.parameters.clone())
                            .unwrap_or_else(|| {
                                self.catalogue.modules[n]
                                    .parameters
                                    .iter()
                                    .map(|(k, p)| (k.clone(), p.default))
                                    .collect()
                            });
                        l.parameters == expected
                    })
                });
        let config = core::parse_config(&r.config, &self.catalogue);
        let saved = if self.target.saved_options {
            config.as_ref().is_ok_and(|all| {
                all.get(n).is_some_and(|opts| {
                    opts.len() == self.target.parameters.len()
                        && opts.iter().all(|(k, v)| {
                            core::integer(v).as_ref() == self.target.parameters.get(k)
                        })
                })
            })
        } else {
            r.config == self.initial.config
        };
        let untouched_config = if self.target.saved_options {
            let before = core::parse_config(&self.initial.config, &self.catalogue)
                .expect("validated initial options");
            config.as_ref().is_ok_and(|all| {
                all.iter()
                    .filter(|(k, _)| *k != n)
                    .eq(before.iter().filter(|(k, _)| *k != n))
            })
        } else {
            saved
        };
        let saved_msg = if let Err(e) = &config {
            format!("Saved options need repair: {e}")
        } else if self.target.saved_options {
            format!(
                "Keep one saved integer option per requested parameter for {n}; these are used on its next insertion, independently of the currently loaded values."
            )
        } else {
            "Keep the supplied saved options unchanged for this runtime-only task.".into()
        };
        let indexed = !self.target.current_index
            || r.index
                == self
                    .catalogue
                    .modules
                    .iter()
                    .filter(|(_, m)| m.kernel_release == self.catalogue.kernel_release)
                    .map(|(n, _)| n.clone())
                    .collect();
        // A saved file alone does not prove the promised next insertion works.
        // Probe a bounded clone, preserving the learner's live state and history.
        let future = if self.target.saved_options {
            let mut probe = r.clone();
            probe
                .execute(&self.catalogue, &format!("modprobe -r {n}"))
                .is_ok_and(|o| o.status == 0)
                && probe
                    .execute(&self.catalogue, &format!("modprobe {n}"))
                    .is_ok_and(|o| o.status == 0)
                && probe
                    .loaded
                    .get(n)
                    .is_some_and(|l| l.parameters == self.target.parameters)
        } else {
            true
        };
        let durable_message = if self.target.saved_options {
            "Future insertion must resolve the supplied driver and dependencies and apply the saved values. A correct running instance alone does not prove the next insertion works."
        } else {
            "The handover needs a rebuilt current-release lookup index, not only a one-off direct insertion."
        };
        vec![(current,effective),(protected,"Preserve every unrelated loaded provider and the supplied/default effective dependency parameters. Do not remove the audit provider to repair the target.".into()),(saved,saved_msg),(untouched_config,"Preserve saved options for every unrelated module; comments and whitespace may change in a configuration task.".into()),(indexed&&future,durable_message.into())]
    }
    fn named_checks(&self, r: &core::Runtime) -> Vec<(bool, String)> {
        let n = &self.target.module;
        let allowed = self.closure();
        let current = r
            .loaded
            .get(n)
            .is_some_and(|l| l.parameters == self.target.parameters);
        let protected = r
            .loaded
            .iter()
            .filter(|(k, _)| !allowed.contains(*k))
            .eq(self
                .initial
                .loaded
                .iter()
                .filter(|(k, _)| !allowed.contains(*k)))
            && allowed.iter().filter(|k| *k != n).all(|k| {
                r.loaded.get(k).is_none_or(|l| {
                    let expected = self
                        .initial
                        .loaded
                        .get(k)
                        .map(|l| l.parameters.clone())
                        .unwrap_or_else(|| {
                            self.catalogue.modules[k]
                                .parameters
                                .iter()
                                .map(|(k, p)| (k.clone(), p.default))
                                .collect()
                        });
                    l.parameters == expected
                })
            });
        let mut rows = vec![
            (
                current,
                format!(
                    "Keep the supplied controller active through {n} with its requested effective parameters; a successful request for another provider is insufficient."
                ),
            ),
            (
                protected,
                "Preserve unrelated loaded providers and the supplied/default dependency settings."
                    .into(),
            ),
        ];
        let cfg = match core::parse_named_config(&r.config, &self.catalogue) {
            Ok(c) => c,
            Err(e) => {
                rows.push((false, format!("Repair the saved binding file: {e}")));
                return rows;
            }
        };
        let original = core::parse_named_config(&self.initial.config, &self.catalogue)
            .expect("validated named initial config");
        let number_values = |configuration: &core::Configuration| {
            configuration
                .options
                .iter()
                .map(|(n, values)| {
                    Some((
                        n.clone(),
                        values
                            .iter()
                            .map(|(k, v)| Some((k.clone(), core::integer(v)?)))
                            .collect::<Option<BTreeMap<_, _>>>()?,
                    ))
                })
                .collect::<Option<BTreeMap<_, _>>>()
        };
        let unchanged_options = number_values(&cfg)
            .zip(number_values(&original))
            .is_some_and(|(a, b)| a == b);
        rows.push((unchanged_options,"Keep the supplied physical-module options unchanged; this handover changes request names, not parameter values.".into()));
        let names = self
            .target
            .request_names
            .iter()
            .map(|n| core::normalized_name(n).expect("validated request name"))
            .collect::<BTreeSet<_>>();
        rows.push((
            cfg.aliases
                .iter()
                .filter(|(k, _)| !names.contains(*k))
                .eq(original.aliases.iter().filter(|(k, _)| !names.contains(*k))),
            "Preserve every unrelated saved binding and leave no additional unrequested aliases."
                .into(),
        ));
        for requested in &self.target.request_names {
            let mut probe = r.clone();
            let removed = self.execute(&mut probe, &format!("modprobe -r {n}"));
            if !removed.is_ok_and(|o| o.status == 0) {
                rows.push((false,format!("Cannot verify the next {requested} request while its target cannot be removed in the test copy. Preserve provider relationships and inspect lsmod.")));
                continue;
            }
            let out = self.execute(&mut probe, &format!("modprobe {requested}"));
            let target = probe
                .loaded
                .get(n)
                .is_some_and(|l| l.parameters == self.target.parameters);
            let outside_unchanged = probe
                .loaded
                .iter()
                .filter(|(k, _)| !allowed.contains(*k))
                .eq(r.loaded.iter().filter(|(k, _)| !allowed.contains(*k)));
            let ok = out.as_ref().is_ok_and(|o| o.status == 0) && target && outside_unchanged;
            let detail=match out{
                Err(e)=>format!("The request was refused: {e}"),
                Ok(o) if o.status!=0=>format!("The request failed: {}",o.stderr.trim()),
                Ok(_) if !target=>"The command returned success but did not activate the required driver with its requested values. A name pointing at the already loaded audit provider can succeed without fixing the controller.".into(),
                Ok(_) if !outside_unchanged=>"The request also changes an unrelated provider. Point this request at the driver itself, not a different module that depends on it.".into(),
                Ok(_)=>"The request activates the required driver and settings without changing unrelated providers.".into(),
            };
            rows.push((ok, format!("Saved request {requested}: {detail}")));
        }
        rows
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Step {
    Command { text: String },
    ExpectedFailure { text: String },
    SaveConfig { text: String },
}
impl Step {
    pub fn action(&self) -> String {
        match self {
            Self::Command { text } | Self::ExpectedFailure { text } => text.clone(),
            Self::SaveConfig { text } => format!(
                "{SAVE_PREFIX}{}",
                serde_json::to_string(text).expect("text encoding")
            ),
        }
    }
    pub fn label(&self) -> String {
        match self {
            Self::Command { text } | Self::ExpectedFailure { text } => text.clone(),
            Self::SaveConfig { .. } => "Edit and save the practice options file".into(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub source_id: String,
    pub assignment: Activity,
    pub example: Activity,
    pub reference: Vec<Step>,
    pub example_steps: Vec<Step>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || !id(&self.source_id) || !bounded(self, 128 * 1024) {
            return Err("Unknown/oversized module task version or source identity.".into());
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if self.assignment.catalogue == self.example.catalogue
            || self.assignment.named() != self.example.named()
            || self.assignment.target.module == self.example.target.module
        {
            return Err(
                "Module practice needs distinct supplied example modules, not assignment state."
                    .into(),
            );
        }
        for (steps, cap) in [(&self.reference, 16), (&self.example_steps, 12)] {
            if steps.is_empty() || steps.len() > cap {
                return Err(
                    "Module teaching needs a bounded complete worked/reference method.".into(),
                );
            }
            for s in steps {
                match s {
                    Step::Command { text } | Step::ExpectedFailure { text }
                        if text.is_empty()
                            || text.len() > core::MAX_COMMAND
                            || text.starts_with('@') =>
                    {
                        return Err(
                            "Reference command must be a real supported literal command.".into(),
                        );
                    }
                    Step::SaveConfig { text } if text.len() > core::MAX_CONFIG => {
                        return Err("Reference options file exceeds its bound.".into());
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        if s.version != 1
            || s.origin != self.source_id
            || s.example_step >= self.example_steps.len()
            || !bounded(s, MAX_SESSION)
        {
            return Err("Module saved work has incompatible source/version/selection/size; original retained.".into());
        }
        s.assignment.bound(Role::Assignment, &self.assignment)?;
        s.practice.bound(Role::Practice, &self.example)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.validate().and_then(|_| self.bound(s)) {
            return vec![(false, e)];
        }
        s.assignment.checks(&self.assignment)
    }
    pub fn validate_reachability(&self) -> Result<(), String> {
        self.validate()?;
        for (practice, steps) in [(false, &self.reference), (true, &self.example_steps)] {
            let mut s = Session::new(self)?;
            for step in steps {
                let out = s.action(self, &step.action(), practice)?;
                let expected_failure = matches!(step, Step::ExpectedFailure { .. });
                if out.starts_with("[Failed") != expected_failure {
                    return Err(format!(
                        "Module reference expected failure={expected_failure}, actual: {out}"
                    ));
                }
            }
            let rows = s.check(self, practice)?;
            if rows.iter().any(|r| !r.0) {
                return Err(format!(
                    "Module reference does not reach {} outcomes: {rows:?}",
                    if practice { "practice" } else { "assignment" }
                ));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub role: Role,
    pub runtime: core::Runtime,
    pub config_draft: String,
    pub command: String,
    pub transcript: Vec<String>,
    pub checked: bool,
}
impl Scope {
    fn new(role: Role, a: &Activity) -> Self {
        Self {
            role,
            config_draft: a.initial.config.clone(),
            runtime: a.initial.clone(),
            command: String::new(),
            transcript: vec![],
            checked: false,
        }
    }
    fn bound(&self, role: Role, a: &Activity) -> Result<(), String> {
        if self.role != role
            || self.command.len() > core::MAX_COMMAND
            || self.config_draft.len() > core::MAX_CONFIG
            || self.config_draft.contains('\0')
            || self.config_draft.lines().count() > 64
            || self.transcript.len() > 40
            || self.transcript.iter().any(|t| t.len() > 8192)
        {
            return Err("Module draft role or field limits differ; original retained.".into());
        }
        a.bound(&self.runtime)
    }
    pub fn checks(&self, a: &Activity) -> Vec<(bool, String)> {
        if !self.checked {
            return vec![(
                false,
                "Not checked · use Check work to inspect current loaded and saved state.".into(),
            )];
        }
        let mut rows = a.checks(&self.runtime);
        rows.push((self.config_draft==self.runtime.config,"Save or revert the options draft before final verification; the loaded driver and saved file are separate states.".into()));
        rows
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub origin: String,
    pub view: View,
    pub example_step: usize,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.source_id.clone(),
            view: View::Introduction,
            example_step: 0,
            assignment: Scope::new(Role::Assignment, &d.assignment),
            practice: Scope::new(Role::Practice, &d.example),
        })
    }
    pub fn shape(&self) -> bool {
        self.version == 1
            && id(&self.origin)
            && self.example_step < 12
            && self.assignment.role == Role::Assignment
            && self.practice.role == Role::Practice
            && bounded(self, MAX_SESSION)
    }
    pub fn run(&mut self, d: &Definition, input: &str, practice: bool) -> Result<String, String> {
        d.validate()?;
        d.bound(self)?;
        let a = if practice { &d.example } else { &d.assignment };
        let scope = if practice {
            &mut self.practice
        } else {
            &mut self.assignment
        };
        let mut next = scope.clone();
        let out = a.execute(&mut next.runtime, input)?;
        if next.runtime != scope.runtime {
            next.checked = false;
        }
        next.bound(scope.role, a)?;
        let display = if out.status == 0 {
            out.display()
        } else {
            format!(
                "[Failed · exit status {}]\n{}\n",
                out.status,
                out.stderr.trim_end()
            )
        };
        *scope = next;
        Ok(display)
    }
    pub fn edit(&mut self, d: &Definition, text: String, practice: bool) -> Result<(), String> {
        d.validate()?;
        d.bound(self)?;
        if text.len() > core::MAX_CONFIG || text.contains('\0') || text.lines().count() > 64 {
            return Err(
                "Options draft exceeds 8KiB/64lines or contains NUL; prior draft retained.".into(),
            );
        }
        let scope = if practice {
            &mut self.practice
        } else {
            &mut self.assignment
        };
        if scope.config_draft != text {
            scope.checked = false;
            scope.config_draft = text;
        }
        Ok(())
    }
    pub fn save(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.validate()?;
        d.bound(self)?;
        let scope = if practice {
            &mut self.practice
        } else {
            &mut self.assignment
        };
        if scope.runtime.config != scope.config_draft {
            scope.runtime.config = scope.config_draft.clone();
            scope.checked = false;
        }
        Ok(())
    }
    pub fn check(&mut self, d: &Definition, practice: bool) -> Result<Vec<(bool, String)>, String> {
        d.validate()?;
        d.bound(self)?;
        let (a, scope) = if practice {
            (&d.example, &mut self.practice)
        } else {
            (&d.assignment, &mut self.assignment)
        };
        scope.checked = true;
        Ok(scope.checks(a))
    }
    /// Bundled reachability codec only. Native terminals never interpret @ actions.
    pub fn action(
        &mut self,
        d: &Definition,
        input: &str,
        practice: bool,
    ) -> Result<String, String> {
        if input == CHECK_ACTION {
            self.check(d, practice)?;
            Ok("Current virtual module state checked.".into())
        } else if let Some(raw) = input.strip_prefix(SAVE_PREFIX) {
            if raw.len() > core::MAX_CONFIG * 6 + 2 {
                return Err("Encoded module options exceed their bound.".into());
            }
            let text =
                serde_json::from_str(raw).map_err(|_| "Invalid module options action data.")?;
            self.edit(d, text, practice)?;
            self.save(d, practice)?;
            Ok("Saved the simulated options file; loaded parameters are unchanged.".into())
        } else {
            self.run(d, input, practice)
        }
    }
}
