//! Source-bound early-userspace tasks; editors and console own virtual data only.
use crate::{boot_entry as boot, initramfs as core, kernel_stage::Store, source_install};
use serde::{Deserialize, Serialize};
type Result<T> = std::result::Result<T, String>;
pub const TASK_LIMIT: usize = 65536;
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
fn text(s: &str, n: usize) -> bool {
    s.len() <= n
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
}
fn optional_source<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Source(#[serde(deserialize_with = "crate::cron::text::<_,4096>")] String);
    Option::<Source>::deserialize(d).map(|v| v.map(|s| s.0))
}
fn optional_path<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Path(#[serde(deserialize_with = "crate::cron::text::<_,128>")] String);
    Option::<Path>::deserialize(d).map(|v| v.map(|p| p.0))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Drivers,
    Handover,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
pub enum Action {
    ConfigDraft {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
    SaveConfig,
    CommandDraft {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    Run {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        input: String,
    },
    EntryDraft {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        filename: String,
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
    SaveEntry,
    Check,
    Reset,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub kind: Kind,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub instructions: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub prefix: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub initial_config: String,
    pub initial_store: Store,
    pub root: boot::Root,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub wanted_release: String,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub entry_filename: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub initial_entry: String,
    #[serde(deserialize_with = "paths")]
    pub preserve_paths: Vec<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,16>")]
    pub reference: Vec<Action>,
}
fn paths<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Path(#[serde(deserialize_with = "crate::cron::text::<_,128>")] String);
    crate::cron_workspace::sequence::<_, Path, 8>(d).map(|v| v.into_iter().map(|p| p.0).collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub output: String,
    pub ok: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub config_draft: String,
    #[serde(deserialize_with = "optional_source")]
    pub saved_config_source: Option<String>,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub command_draft: String,
    pub store: Store,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub entry_filename: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub entry_draft: String,
    #[serde(deserialize_with = "optional_path")]
    pub saved_entry: Option<String>,
    pub checked: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Line,96>")]
    pub log: Vec<Line>,
}
impl Work {
    pub fn new(t: &Task) -> Self {
        let selected = entry_path(t, &t.entry_filename).ok().filter(|p| {
            t.initial_store
                .entries
                .iter()
                .any(|e| &e.path == p && e.source == t.initial_entry)
        });
        Self {
            config_draft: t.initial_config.clone(),
            saved_config_source: core::Config::parse(&t.initial_config)
                .ok()
                .map(|_| t.initial_config.clone()),
            command_draft: String::new(),
            store: t.initial_store.clone(),
            entry_filename: t.entry_filename.clone(),
            entry_draft: t.initial_entry.clone(),
            saved_entry: selected,
            checked: false,
            log: vec![],
        }
    }
    pub fn saved_config(&self) -> Result<core::Config> {
        core::Config::parse(
            self.saved_config_source
                .as_deref()
                .ok_or("Save a supported configuration fragment before generating an image.")?,
        )
    }
}
fn entry_path(t: &Task, name: &str) -> Result<String> {
    if name.len() > 48
        || !name.ends_with(".conf")
        || name.contains('/')
        || !boot::path(&format!("/{name}"))
    {
        return Err("Use one .conf entry filename without directories or .. components.".into());
    }
    source_install::join(&t.prefix, &format!("boot/loader/entries/{name}"))
}
fn same_file(a: &Store, b: &Store, path: &str) -> bool {
    if let Some(old) = a.images.iter().find(|f| f.path == path) {
        return b.images.iter().any(|f| f == old);
    }
    if let Some(old) = a.modules.iter().find(|f| f.path == path) {
        return b.modules.iter().any(|f| f == old);
    }
    if let Some(old) = a.initramfs.iter().find(|f| f.path == path) {
        return b.initramfs.iter().any(|f| f == old);
    }
    if let Some(old) = a.entries.iter().find(|f| f.path == path) {
        return b.entries.iter().any(|f| f == old);
    }
    false
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub detail: String,
}
fn check(label: &str, result: Result<String>) -> Check {
    match result {
        Ok(detail) => Check {
            passed: true,
            label: label.into(),
            detail,
        },
        Err(detail) => Check {
            passed: false,
            label: label.into(),
            detail,
        },
    }
}
impl Task {
    pub fn validate_work(&self, w: &Work) -> Result<()> {
        self.initial_store.validate()?;
        w.store.validate()?;
        if w.store.images != self.initial_store.images
            || w.store.modules != self.initial_store.modules
        {
            return Err("Supplied kernel images and prepared module trees are fixed inputs; keep original work when those facts differ.".into());
        }
        if !text(&w.config_draft, 4096)
            || !text(&w.command_draft, 2048)
            || !text(&w.entry_filename, 48)
            || !text(&w.entry_draft, 4096)
            || w.saved_entry.as_ref().is_some_and(|p| !boot::path(p))
            || w.log.len() > 96
            || w.log
                .iter()
                .any(|l| !text(&l.input, 2048) || !text(&l.output, 8192))
        {
            return Err("Keep saved editors, command history and selected entry within their bounded plain-text limits.".into());
        }
        if self.kind == Kind::Drivers
            && (w.store.entries != self.initial_store.entries
                || w.entry_filename != self.entry_filename
                || w.entry_draft != self.initial_entry)
        {
            return Err("This driver-repair activity keeps the supplied entry fixed; its original selected file is retained.".into());
        }
        if let Some(saved) = &w.saved_config_source {
            if !text(saved, 4096) {
                return Err("Saved configuration exceeds its text limit.".into());
            }
            core::Config::parse(saved)?;
        }
        let boot_root = format!("{}/", source_install::join(&self.prefix, "boot")?);
        let entries = format!("{boot_root}loader/entries/");
        for f in &w.store.initramfs {
            if !f
                .path
                .strip_prefix(&boot_root)
                .is_some_and(|leaf| !leaf.contains('/'))
                || !self
                    .initial_store
                    .images
                    .iter()
                    .any(|i| i.build.same_snapshot(&f.image.image_build))
                || f.image.module_build.as_ref().is_some_and(|m| {
                    !self
                        .initial_store
                        .modules
                        .iter()
                        .any(|t| t.build.same_snapshot(m))
                })
            {
                return Err("An early-userspace record needs a canonical supplied boot filename and prepared source/module identity; host or unknown builds are not imported.".into());
            }
        }
        for f in &w.store.entries {
            let name = f
                .path
                .strip_prefix(&entries)
                .ok_or("Keep entries in the supplied virtual loader directory.")?;
            entry_path(self, name)?;
        }
        if w.saved_entry
            .iter()
            .any(|p| !w.store.entries.iter().any(|f| &f.path == p))
        {
            return Err("The saved selected entry must exist in this virtual store.".into());
        }
        Ok(())
    }
    pub fn apply(&self, w: &mut Work, a: &Action) -> Result<String> {
        self.validate_work(w)?;
        match a {
            Action::ConfigDraft { text: value } => {
                if !text(value, 4096) {
                    return Err("Keep the configuration fragment within 4 KiB.".into());
                }
                if *value != w.config_draft {
                    w.config_draft = value.clone();
                    w.checked = false;
                }
                Ok("Draft retained; generation still reads the saved configuration.".into())
            }
            Action::SaveConfig => {
                core::Config::parse(&w.config_draft)?;
                if w.saved_config_source.as_ref() != Some(&w.config_draft) {
                    w.checked = false;
                }
                w.saved_config_source = Some(w.config_draft.clone());
                Ok("Saved the virtual driver-selection fragment.".into())
            }
            Action::CommandDraft { text: value } => {
                if !text(value, 2048) {
                    return Err("Keep one command draft within 2,048 bytes.".into());
                }
                w.command_draft = value.clone();
                Ok("Command draft retained; it has not run.".into())
            }
            Action::Run { input } => {
                if !text(input, 2048) {
                    return Err("Keep one console operation within 2,048 bytes.".into());
                }
                let before = w.store.clone();
                let result = match core::parse(input) {
                    Ok(core::Command::Build { .. }) => w.saved_config().and_then(|config| {
                        core::execute(&self.prefix, &config, &mut w.store, input)
                    }),
                    _ => core::execute(&self.prefix, &core::Config::default(), &mut w.store, input),
                };
                if w.store != before {
                    w.checked = false;
                }
                let (ok, output) = match &result {
                    Ok(v) => (true, v.clone()),
                    Err(e) => (false, e.clone()),
                };
                if output.len() > 8192 {
                    return Err("The modeled console result exceeded its 8 KiB limit.".into());
                }
                if w.log.len() == 96 {
                    w.log.remove(0);
                }
                w.log.push(Line {
                    input: input.clone(),
                    output,
                    ok,
                });
                result
            }
            Action::EntryDraft {
                filename,
                text: value,
            } => {
                if self.kind != Kind::Handover {
                    return Err("This activity keeps its supplied entry fixed; repair driver selection and the selected image.".into());
                }
                if !text(filename, 48) || !text(value, 4096) {
                    return Err(
                        "Keep the entry filename within48characters and text within4KiB.".into(),
                    );
                }
                if filename != &w.entry_filename || value != &w.entry_draft {
                    w.entry_filename = filename.clone();
                    w.entry_draft = value.clone();
                    w.checked = false;
                }
                Ok("Entry draft retained; save it before verification.".into())
            }
            Action::SaveEntry => {
                if self.kind != Kind::Handover {
                    return Err("This activity uses its supplied fixed entry.".into());
                }
                let path = entry_path(self, &w.entry_filename)?;
                if self.preserve_paths.contains(&path) {
                    return Err(
                        "Use a different entry filename; the working fallback entry is protected."
                            .into(),
                    );
                }
                let old = w.store.clone();
                let path = w
                    .store
                    .entry(&self.prefix, &w.entry_filename, &w.entry_draft)?;
                if old != w.store || w.saved_entry.as_ref() != Some(&path) {
                    w.checked = false;
                }
                w.saved_entry = Some(path);
                Ok("Saved the selected virtual loader entry.".into())
            }
            Action::Check => {
                w.checked = true;
                Ok("Checked actual saved choices, selected files, boot consequence and protected fallback.".into())
            }
            Action::Reset => {
                *w = Work::new(self);
                Ok("Reset this workspace only; its other scope is unchanged.".into())
            }
        }
    }
    fn selected<'a>(&self, w: &'a Work) -> Result<&'a crate::kernel_stage::EntryFile> {
        let path = w
            .saved_entry
            .as_ref()
            .ok_or("Save a candidate entry before checking.")?;
        if entry_path(self, &w.entry_filename)? != *path {
            return Err(
                "The entry filename is an unsaved draft. Save entry before checking it.".into(),
            );
        }
        let entry = w
            .store
            .entries
            .iter()
            .find(|e| &e.path == path)
            .ok_or("The selected virtual entry is missing.")?;
        if entry.source != w.entry_draft {
            return Err(
                "The entry text has unsaved changes. Save entry before checking it.".into(),
            );
        }
        Ok(entry)
    }
    pub fn checks(&self, w: &Work) -> Vec<Check> {
        if let Err(e) = self.validate_work(w) {
            return vec![check("Saved workspace", Err(e))];
        }
        let mut checks = vec![check(
            "Saved driver choices",
            if w.saved_config_source.as_ref() == Some(&w.config_draft) {
                w.saved_config().map(|_| {
                    "The current fragment is saved; generation reads these exact choices.".into()
                })
            } else {
                Err("Configuration changes are still a draft. Save the fragment, regenerate the selected image, then check again.".into())
            },
        )];
        let selected = self.selected(w);
        let parsed = selected
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|f| boot::Entry::parse(&f.source).map_err(|e| e.to_string()));
        let target=parsed.as_ref().map_err(Clone::clone).and_then(|e|{
            let path=format!("{}{}",source_install::join(&self.prefix,"boot")?,e.linux);
            let kernel=w.store.images.iter().find(|i|i.path==path).ok_or("The selected linux path has no supplied kernel image.")?;
            if kernel.build.release!=self.wanted_release{return Err(format!("The entry selects {} instead of target {}. A working fallback is not the maintenance target.",kernel.build.release,self.wanted_release));}Ok(format!("The selected entry uses supplied target {}.",self.wanted_release))
        });
        checks.push(check("Target kernel selection", target));
        let current=parsed.as_ref().map_err(Clone::clone).and_then(|e|{
            if e.initrd.is_empty(){return Err("Select the target early-userspace image in an initrd line; this UUID-root exercise requires it.".into());}
            let config=w.saved_config()?;
            for name in &e.initrd {
                let path=format!("{}{}",source_install::join(&self.prefix,"boot")?,name);
                let image=w.store.initramfs.iter().find(|i|i.path==path).ok_or_else(||format!("No generated virtual image exists at {path}. Generate it or select an existing matching image."))?;
                if image.image.image_build.release!=self.wanted_release{return Err(format!("{name} contains drivers for {}, not {}. Filenames do not prove kernel identity; inspect it and use the intended --kver.",image.image.image_build.release,self.wanted_release));}
                let expected=image.image.image_build.config.loadable_inventory().difference(&config.omitted).cloned().collect::<std::collections::BTreeSet<_>>();
                if image.image.included!=expected{return Err("The selected image differs from the saved driver choices. Saving a fragment does not rebuild its image; regenerate, inspect and retry.".into());}
            }Ok("Selected image records match the target release and current saved driver choices.".into())
        });
        checks.push(check("Current target early userspace", current));
        let boot=selected.map_err(|e|e.to_string()).and_then(|e|{
            let report=w.store.test_entry(&self.prefix,&e.path,self.root.clone())?;
            if report.passed(){Ok("The modeled entry reaches the supplied root and init handoff; no machine boot occurred.".into())}else{Err(report.stages.into_iter().find(|s|!s.passed).map(|s|s.detail).unwrap_or_else(||"The modeled root handoff did not complete.".into()))}
        });
        checks.push(check("Candidate root handoff", boot));
        let changed = self
            .preserve_paths
            .iter()
            .filter(|p| !same_file(&self.initial_store, &w.store, p))
            .cloned()
            .collect::<Vec<_>>();
        checks.push(check("Working fallback preserved",if changed.is_empty(){Ok("Every protected supplied file remains unchanged.".into())}else{Err(format!("Protected files changed: {}. Restore the fallback and target the maintenance image explicitly.",changed.join(", ")))}));
        checks
    }
    pub fn solved(&self, w: &Work) -> bool {
        w.checked && self.checks(w).iter().all(|c| c.passed)
    }
    pub fn validate(&self) -> Result<()> {
        if !id(&self.id)
            || self.instructions.trim().is_empty()
            || !text(&self.instructions, 1600)
            || !source_install::directory(&self.prefix)
            || !text(&self.initial_config, 4096)
            || self.wanted_release.is_empty()
            || self.wanted_release.len() > 64
            || self.reference.is_empty()
            || self.reference.len() > 16
            || !matches!(self.reference.last(), Some(Action::Check))
            || self.preserve_paths.is_empty()
            || self.preserve_paths.len() > 8
        {
            return Err("Supply a bounded named early-userspace task, target, protected fallback and reachable reference ending in Check.".into());
        }
        self.initial_store
            .inventory(&self.prefix, self.root.clone())?;
        let wanted = self
            .initial_store
            .images
            .iter()
            .filter(|i| i.build.release == self.wanted_release)
            .count();
        if wanted != 1
            || self
                .root
                .required_drivers
                .iter()
                .map(String::as_str)
                .collect::<std::collections::BTreeSet<_>>()
                != ["nvme_core", "nvme", "ext4", "jbd2", "mbcache"]
                    .into_iter()
                    .collect()
            || self.root.required_drivers.len() != 5
        {
            return Err("This finite task needs one supplied target and the declared NVMe/ext4 root-driver graph.".into());
        }
        let mut paths = std::collections::BTreeSet::new();
        let mut fallback = false;
        for path in &self.preserve_paths {
            if !boot::path(path)
                || !paths.insert(path)
                || !same_file(&self.initial_store, &self.initial_store, path)
            {
                return Err(
                    "Protect distinct existing canonical virtual files, not unknown paths.".into(),
                );
            }
            if self.initial_store.entries.iter().any(|e| &e.path == path) {
                if !self
                    .initial_store
                    .test_entry(&self.prefix, path, self.root.clone())?
                    .passed()
                {
                    return Err(
                        "A protected fallback entry must actually pass the supplied boot model."
                            .into(),
                    );
                }
                fallback = true;
            }
        }
        if !fallback {
            return Err("Supply at least one genuinely working protected fallback entry.".into());
        }
        let mut w = Work::new(self);
        self.validate_work(&w)?;
        if w.saved_entry.is_none() {
            return Err("Supply the candidate's existing entry and exact initial text before practice begins.".into());
        }
        if self.kind == Kind::Drivers
            && w.saved_entry
                .as_ref()
                .is_some_and(|p| self.preserve_paths.contains(p))
        {
            return Err("The repair target must be separate from the protected fallback.".into());
        }
        w.checked = true;
        if self.solved(&w) {
            return Err("The starter must leave meaningful work unfinished.".into());
        }
        w.checked = false;
        for a in &self.reference {
            if matches!(a, Action::Reset | Action::CommandDraft { .. }) {
                return Err("Use supported worked operations, not reset/draft-only actions, in the reference.".into());
            }
            self.apply(&mut w, a)?;
        }
        if !self.solved(&w) {
            return Err(
                "The authored reference does not reach every actual early-userspace outcome."
                    .into(),
            );
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > TASK_LIMIT {
            return Err("The task exceeds its64KiB persisted source limit.".into());
        }
        Ok(())
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
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || !id(&self.source_id)
            || self.source_id != self.assignment.id
            || self.assignment.id == self.example.id
            || self.assignment.kind != self.example.kind
            || self.assignment.initial_store == self.example.initial_store
        {
            return Err("Keep a versioned named assignment and the same activity in a separate different-input example.".into());
        }
        self.assignment.validate()?;
        self.example.validate()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub source_task: String,
    pub work: Work,
}
fn canonical(t: &Task) -> Result<String> {
    let s = serde_json::to_string(t).map_err(|e| e.to_string())?;
    if s.len() > TASK_LIMIT {
        return Err("Task source exceeds64KiB.".into());
    }
    Ok(s)
}
impl Scope {
    pub fn new(t: &Task) -> Result<Self> {
        t.validate()?;
        Ok(Self {
            source_task: canonical(t)?,
            work: Work::new(t),
        })
    }
    pub fn bind(&self, t: &Task) -> Result<()> {
        if self.source_task != canonical(t)? {
            return Err("Saved early-userspace facts or requirements differ from this task; keep the original work.".into());
        }
        t.validate_work(&self.work)
    }
    pub fn validate_shape(&self) -> Result<()> {
        if self.source_task.len() > TASK_LIMIT {
            return Err("Saved task source exceeds64KiB.".into());
        }
        let t: Task = serde_json::from_str(&self.source_task)
            .map_err(|e| format!("Malformed saved early-userspace task: {e}"))?;
        t.validate()?;
        self.bind(&t)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub view: crate::script_teaching::View,
    pub assignment: Scope,
    pub example: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            version: 1,
            source_id: d.source_id.clone(),
            view: crate::script_teaching::View::Introduction,
            assignment: Scope::new(&d.assignment)?,
            example: Scope::new(&d.example)?,
        })
    }
    pub fn bind(&self, d: &Definition) -> Result<()> {
        d.validate()?;
        if self.version != 1 || self.source_id != d.source_id {
            return Err("Saved early-userspace session belongs to another lesson/version; retain original work.".into());
        }
        self.assignment.bind(&d.assignment)?;
        self.example.bind(&d.example)
    }
    pub fn validate_shape(&self) -> Result<()> {
        if self.version != 1 || !id(&self.source_id) {
            return Err("Use a named supported early-userspace session version.".into());
        }
        self.assignment.validate_shape()?;
        self.example.validate_shape()?;
        let assignment =
            serde_json::from_str(&self.assignment.source_task).map_err(|e| e.to_string())?;
        let example = serde_json::from_str(&self.example.source_task).map_err(|e| e.to_string())?;
        self.bind(&Definition {
            version: 1,
            source_id: self.source_id.clone(),
            assignment,
            example,
        })
    }
    pub fn apply(&mut self, d: &Definition, practice: bool, a: &Action) -> Result<String> {
        self.bind(d)?;
        let (t, w) = if practice {
            (&d.example, &mut self.example.work)
        } else {
            (&d.assignment, &mut self.assignment.work)
        };
        t.apply(w, a)
    }
}
