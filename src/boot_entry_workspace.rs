//! Source-held early boot tasks. Drafts and tested observations are never authority.
use crate::{boot_entry as core, script_teaching::View};
use serde::{Deserialize, Serialize};
pub const PREFIX: &str = "BOOT ENTRY WORK\n";
/// Source-held requirements for the opt-in literal menu task. Never saved authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuPolicy {
    #[serde(deserialize_with = "menu_ids")]
    pub preserve: Vec<String>,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub selected_id: String,
}
/// Four-key persistent settings over fixed, supplied templates. Never executable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratorPolicy {
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub templates: String,
    #[serde(deserialize_with = "menu_ids")]
    pub normal_ids: Vec<String>,
    pub timeout: i32,
    pub normal_quiet: bool,
    pub recovery_quiet: bool,
}
fn menu_ids<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,64>")] String);
    crate::cron_workspace::sequence::<_, Text, 4>(d).map(|v| v.into_iter().map(|t| t.0).collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub instructions: String,
    pub inventory: core::Inventory,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub menu: Option<MenuPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generator: Option<GeneratorPolicy>,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub entry_path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub starter: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub reference: String,
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub required_root: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub required_release: String,
    pub require_read_only: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub required_target: String,
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn source_shape(s: &str) -> bool {
    s.len() <= core::MAX_SOURCE && !s.contains('\0')
}
impl Task {
    pub fn shape(&self) -> Result<(), String> {
        self.inventory.validate().map_err(|e| e.to_string())?;
        let proper_entry_path = if self.generator.is_some() {
            self.entry_path == "/etc/default/grub"
        } else if self.menu.is_some() {
            self.entry_path.starts_with("/grub/learning-") && self.entry_path.ends_with(".cfg")
        } else {
            self.entry_path.starts_with("/loader/entries/") && self.entry_path.ends_with(".conf")
        };
        if !id(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 2048
            || !core::path(&self.entry_path)
            || !proper_entry_path
            || !source_shape(&self.starter)
            || !source_shape(&self.reference)
            || !self
                .inventory
                .roots
                .iter()
                .any(|r| r.uuid.eq_ignore_ascii_case(&self.required_root))
            || !self
                .inventory
                .kernels
                .iter()
                .any(|k| k.release == self.required_release)
            || !matches!(
                self.required_target.as_str(),
                "multi-user.target" | "rescue.target"
            )
        {
            return Err(
                "Invalid supplied boot-entry identity, path, inputs or requested handoff.".into(),
            );
        }
        if let Some(policy) = &self.generator {
            if self.menu.is_some() || !(-1..=300).contains(&policy.timeout) {
                return Err("Use one supplied configuration format and a bounded timeout.".into());
            }
            let generated =
                crate::grub_settings::generate("", &policy.templates, &policy.normal_ids)
                    .map_err(|e| e.to_string())?;
            if generated.menu.items.len() <= policy.normal_ids.len() {
                return Err("The supplied generator must retain a separate recovery entry.".into());
            }
        }
        if let Some(policy) = &self.menu {
            let starter =
                crate::grub_menu::Menu::parse(&self.starter).map_err(|e| e.to_string())?;
            if policy.preserve.is_empty()
                || policy.preserve.len() > 4
                || policy
                    .preserve
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != policy.preserve.len()
                || policy
                    .preserve
                    .iter()
                    .any(|id| !starter.items.iter().any(|e| &e.id == id))
                || (!policy.selected_id.is_empty()
                    && !starter.items.iter().any(|e| e.id == policy.selected_id))
            {
                return Err("Menu policy must name unique supplied entries; keep at least one protected path.".into());
            }
        }
        Ok(())
    }
    pub fn report(&self, source: &str) -> Result<core::Report, core::Error> {
        if let Some(policy) = &self.generator {
            let generated =
                crate::grub_settings::generate(source, &policy.templates, &policy.normal_ids)?;
            crate::grub_menu::test(&self.inventory, &generated.menu_source)
        } else if self.menu.is_some() {
            crate::grub_menu::test(&self.inventory, source)
        } else {
            core::test(&self.inventory, source)
        }
    }
    pub fn evaluate(&self, source: &str) -> Vec<(bool, String)> {
        match self.report(source) {
            Err(e) => vec![(false, format!("{:?}: {}", e.kind, e.message))],
            Ok(r) => {
                let mut checks = r
                    .stages
                    .iter()
                    .map(|s| (s.passed, format!("{}: {}", s.title, s.detail)))
                    .collect::<Vec<_>>();
                checks.push((
                    r.passed(),
                    if r.passed() {
                        "Modeled init handoff reached."
                    } else {
                        "The handoff is blocked; repair the first failed stage and retry."
                    }
                    .into(),
                ));
                checks.push((
                    r.requested.root.eq_ignore_ascii_case(&self.required_root),
                    "Configured root=UUID= must identify the requested system filesystem.".into(),
                ));
                checks.push((
                    r.release.as_deref() == Some(self.required_release.as_str()),
                    format!(
                        "Select a kernel with the required release {}.",
                        self.required_release
                    ),
                ));
                checks.push((
                    r.read_only == self.require_read_only,
                    format!(
                        "The requested initial root mode is {}.",
                        if self.require_read_only {
                            "read-only (ro)"
                        } else {
                            "read/write (rw)"
                        }
                    ),
                ));
                checks.push((
                    r.requested.target == self.required_target,
                    format!(
                        "Configured startup request must be {}.",
                        self.required_target
                    ),
                ));
                if let Some(policy) = &self.menu {
                    match (crate::grub_menu::Menu::parse(&self.starter), crate::grub_menu::Menu::parse(source)) {
                        (Ok(original),Ok(actual)) => {
                            if !policy.selected_id.is_empty() {
                                checks.push((actual.selected().is_ok_and(|e|e.id==policy.selected_id),
                                    format!("Select the requested entry ID {}; another bootable entry does not replace this task.",policy.selected_id)));
                            }
                            if policy.selected_id.is_empty() {
                                checks.push((actual.selected().is_ok_and(|chosen|original.items.iter().any(|e|e.id==chosen.id)),
                                    "Select one of the supplied menu entries; a newly added path is not this fallback exercise.".into()));
                            }
                            for id in &policy.preserve {
                                let changed=actual.changed_fields(&original,id);
                                checks.push((changed.is_empty(), if changed.is_empty() {
                                    format!("Supplied entry ID {id} is preserved.")
                                } else {
                                    format!("Preserve supplied entry ID {id}: {}. Restore that entry and edit the requested path instead.",changed.join("; "))
                                }));
                            }
                        }
                        _ => checks.push((false,"The supplied or edited menu cannot be compared; restore valid literal syntax and retry.".into())),
                    }
                }
                if let Some(policy) = &self.generator {
                    match crate::grub_settings::generate(
                        source,
                        &policy.templates,
                        &policy.normal_ids,
                    ) {
                        Err(e) => checks.push((false, e.to_string())),
                        Ok(generated) => {
                            checks.push((
                                generated.timeout == policy.timeout,
                                format!(
                                    "Generated menu timeout must be {} seconds; actual is {}.",
                                    policy.timeout, generated.timeout
                                ),
                            ));
                            for item in &generated.menu.items {
                                let normal = policy.normal_ids.contains(&item.id);
                                let expected_quiet = if normal {
                                    policy.normal_quiet
                                } else {
                                    policy.recovery_quiet
                                };
                                let actual_quiet = item.entry.options.iter().any(|v| v == "quiet");
                                let trace = core::trace_entry(&self.inventory, &item.entry);
                                checks.push((trace.as_ref().is_ok_and(|r|r.passed()&&r.root.as_deref().is_some_and(|v|v.eq_ignore_ascii_case(&self.required_root))&&r.read_only==self.require_read_only),format!("Generated entry {} must reach the supplied system with initial root mode {}. Inspect common arguments if several entries fail.",item.id,if self.require_read_only{"ro"}else{"rw"})));
                                checks.push((actual_quiet==expected_quiet,format!("Generated {} entry {}: quiet must be {}; actual is {}. GRUB_CMDLINE_LINUX applies to both classes; GRUB_CMDLINE_LINUX_DEFAULT only to normal entries.",if normal{"normal"}else{"recovery"},item.id,if expected_quiet{"present"}else{"absent"},if actual_quiet{"present"}else{"absent"})));
                            }
                        }
                    }
                }
                checks
            }
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        if self.evaluate(&self.reference).iter().any(|c| !c.0) {
            return Err(
                "The worked boot-entry repair does not reach every requested outcome.".into(),
            );
        }
        if self.evaluate(&self.starter).iter().all(|c| c.0) {
            return Err("The boot-entry starter must retain meaningful independent work.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.initial.menu.is_some() != self.example.menu.is_some()
            || self.initial.generator.is_some() != self.example.generator.is_some()
            || self.initial.id == self.example.id
            || self
                .initial
                .required_root
                .eq_ignore_ascii_case(&self.example.required_root)
            || self.initial.reference == self.example.reference
        {
            return Err(
                "Use version1 and genuinely separate boot-entry practice facts/drafts.".into(),
            );
        }
        self.initial.validate()?;
        self.example.validate()
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = s.bound(self) {
            return vec![(false, e)];
        }
        s.assignment.checks(&self.initial)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub source: String,
    #[serde(default, deserialize_with = "checked_source")]
    pub checked: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated: Option<GeneratedRecord>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedRecord {
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub settings: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub menu: String,
    pub timeout: i32,
}
fn checked_source<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,4096>")] String);
    Option::<Text>::deserialize(d).map(|t| t.map(|t| t.0))
}
impl Draft {
    pub fn new(t: &Task) -> Self {
        Self {
            source: t.starter.clone(),
            checked: None,
            generated: None,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if !source_shape(&self.source)
            || self
                .checked
                .as_ref()
                .is_some_and(|s| !source_shape(s) || s != &self.source)
        {
            return Err(
                "Saved boot-entry text or observation is incompatible; original retained.".into(),
            );
        }
        if self.generated.as_ref().is_some_and(|g| {
            !source_shape(&g.settings) || !source_shape(&g.menu) || !(-1..=300).contains(&g.timeout)
        }) {
            return Err("Saved generated menu exceeds its bounds; original retained.".into());
        }
        Ok(())
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        self.validate()?;
        if let Some(record) = &self.generated {
            let policy = t
                .generator
                .as_ref()
                .ok_or("This task has no generated menu; original retained.")?;
            let actual = crate::grub_settings::generate(
                &record.settings,
                &policy.templates,
                &policy.normal_ids,
            )
            .map_err(|e| e.to_string())?;
            if actual.menu_source != record.menu || actual.timeout != record.timeout {
                return Err("Saved generated menu does not match its supplied settings and templates; original retained.".into());
            }
        }
        Ok(())
    }
    pub fn generate(&mut self, t: &Task) -> Result<(), String> {
        self.bound(t)?;
        let policy = t
            .generator
            .as_ref()
            .ok_or("Only a persistent-settings task has a Generate action.")?;
        let actual =
            crate::grub_settings::generate(&self.source, &policy.templates, &policy.normal_ids)
                .map_err(|e| e.to_string())?;
        if !source_shape(&actual.menu_source) {
            return Err("Generated menu exceeds 4 KiB; no output replaced.".into());
        }
        self.generated = Some(GeneratedRecord {
            settings: self.source.clone(),
            menu: actual.menu_source,
            timeout: actual.timeout,
        });
        self.checked = None;
        Ok(())
    }
    pub fn report(&self, t: &Task) -> Result<core::Report, core::Error> {
        if t.generator.is_some() {
            let record=self.generated.as_ref().filter(|r|r.settings==self.source).ok_or_else(||core::Error{kind:core::ErrorKind::Invalid,message:"Generate the current settings before testing. An older generated menu is not updated by editing this file.".into()})?;
            self.bound(t).map_err(|message| core::Error {
                kind: core::ErrorKind::Invalid,
                message,
            })?;
            crate::grub_menu::test(&t.inventory, &record.menu)
        } else {
            t.report(&self.source)
        }
    }
    pub fn edit(&mut self, source: String) -> Result<(), String> {
        if !source_shape(&source) {
            return Err("Keep the entry within4KiB with no NUL character.".into());
        }
        if source != self.source {
            self.source = source;
            self.checked = None;
        }
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<Vec<(bool, String)>, String> {
        self.bound(t)?;
        t.shape()?;
        self.checked = Some(self.source.clone());
        Ok(self.checks(t))
    }
    pub fn checks(&self, t: &Task) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(t) {
            return vec![(false, e)];
        }
        if self.checked.is_none() {
            return vec![(
                false,
                if t.generator.is_some() {
                    "Not tested for the current settings. Generate, then Test generated menu."
                } else if t.menu.is_some() {
                    "Not tested for the current menu. Use Test menu after editing."
                } else {
                    "Not tested for the current entry. Use Test entry after editing."
                }
                .into(),
            )];
        }
        if t.generator.is_some()
            && let Err(e) = self.report(t)
        {
            return vec![(false, e.to_string())];
        }
        t.evaluate(&self.source)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub context: String,
    pub view: View,
    pub assignment: Draft,
    pub practice: Draft,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            context: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Draft::new(&d.initial),
            practice: Draft::new(&d.example),
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || !id(&self.context) {
            return Err("Invalid saved boot-entry version/identity.".into());
        }
        self.assignment.validate()?;
        self.practice.validate()
    }
    pub fn bound(&self, d: &Definition) -> Result<(), String> {
        self.validate()?;
        if self.context != d.initial.id {
            return Err("Boot-entry work belongs to another task; original retained.".into());
        }
        d.initial.shape()?;
        d.example.shape()?;
        self.assignment.bound(&d.initial)?;
        self.practice.bound(&d.example)
    }
    pub fn execute_reference(&mut self, d: &Definition, raw: &str) -> Result<String, String> {
        self.bound(d)?;
        let raw = raw.strip_prefix(PREFIX).ok_or(core::HELP)?;
        if raw.len() > 32768 {
            return Err("Boot-entry work envelope exceeds its bound.".into());
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Work {
            #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
            source: String,
            #[serde(default)]
            generate: bool,
        }
        let w: Work = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid boot-entry work envelope: {e}"))?;
        let mut next = self.assignment.clone();
        next.edit(w.source)?;
        if w.generate {
            next.generate(&d.initial)?;
        }
        let checks = next.check(&d.initial)?;
        let passed = checks.iter().filter(|c| c.0).count();
        self.assignment = next;
        Ok(format!(
            "Boot-entry outcomes: {passed}/{} match",
            checks.len()
        ))
    }
}
fn admitted(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::Goal::BootEntry { definition },
        crate::engine::State::BootEntry { simulation },
    ) = (&s.goal, &s.initial)
    else {
        return Err("Boot-entry task/source mismatch.".into());
    };
    let fresh = Session {
        version: 1,
        context: definition.initial.id.clone(),
        view: View::Introduction,
        assignment: Draft::new(&definition.initial),
        practice: Draft::new(&definition.example),
    };
    if s.track != "LPIC-1"
        || definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != fresh
    {
        return Err("Boot-entry source, fresh session, track and objective must match.".into());
    }
    Ok(definition)
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let d = admitted(s)?;
    d.validate()?;
    Ok(d)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = admitted(s)?;
    let crate::engine::State::BootEntry { simulation } = &a.state else {
        return Err("Boot-entry work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible boot-entry metadata; original retained.".into());
    }
    simulation.bound(d)
}
