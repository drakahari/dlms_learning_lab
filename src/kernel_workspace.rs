//! Source-bound task work; examples and assignments own independent values.
//! All actions use typed model/virtual-store operations, never host execution.
use crate::{boot_entry, kernel_build as core, kernel_commands, kernel_stage, source_install};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, String>;
pub const MAX_TASK_BYTES: usize = 65536;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub detail: String,
}

fn identity(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && !matches!(s, "." | "..")
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn text(s: &str, cap: usize) -> bool {
    s.len() <= cap
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
}
fn optional_path<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Path(#[serde(deserialize_with = "crate::cron::text::<_,128>")] String);
    Option::<Path>::deserialize(d).map(|v| v.map(|p| p.0))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub release: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub title: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Image,
    Boot,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
pub enum Action {
    Source {
        index: usize,
    },
    Config {
        #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
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
    StageImage,
    Initramfs {
        #[serde(deserialize_with = "crate::cron::text::<_,128>")]
        image_path: String,
    },
    WorkingInitramfs,
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
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Source,2>")]
    pub sources: Vec<Source>,
    pub initial_source: usize,
    pub wanted_source: usize,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub initial_config: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub stage_root: String,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub initial_entry_filename: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub initial_entry: String,
    pub initial_store: kernel_stage::Store,
    pub root: Option<boot_entry::Root>,
    pub require_initrd_support: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,24>")]
    pub reference: Vec<Action>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub output: String,
    pub ok: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub source: usize,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub config_draft: String,
    pub saved_config: Option<core::Config>,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub command_draft: String,
    pub outputs: core::Artifacts,
    pub store: kernel_stage::Store,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub entry_filename: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub entry_draft: String,
    #[serde(default, deserialize_with = "optional_path")]
    pub saved_entry: Option<String>,
    pub checked: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Line,128>")]
    pub log: Vec<Line>,
}

impl Work {
    pub fn new(t: &Task) -> Self {
        Self {
            source: t.initial_source,
            config_draft: t.initial_config.clone(),
            saved_config: core::Config::parse(&t.initial_config).ok(),
            command_draft: String::new(),
            outputs: core::Artifacts::default(),
            store: t.initial_store.clone(),
            entry_filename: t.initial_entry_filename.clone(),
            entry_draft: t.initial_entry.clone(),
            saved_entry: None,
            checked: false,
            log: Vec::new(),
        }
    }
    fn line(&mut self, input: &str, output: &str, ok: bool) {
        if self.log.len() == 128 {
            self.log.remove(0);
        }
        self.log.push(Line {
            input: input.into(),
            output: output.into(),
            ok,
        });
    }
}

impl Task {
    fn record(&self, r: &core::Build) -> Result<()> {
        let source = self
            .sources
            .iter()
            .find(|s| s.id == r.source_id)
            .ok_or("Build record names an unknown supplied source.")?;
        if !r.current_for(&source.id, &source.release, &r.config) {
            return Err(
                "Build record differs from its supplied source/configuration identity.".into(),
            );
        }
        Ok(())
    }

    pub fn validate_work(&self, w: &Work) -> Result<()> {
        if w.source >= self.sources.len()
            || !text(&w.config_draft, 8192)
            || !text(&w.entry_draft, 4096)
            || !text(&w.entry_filename, 48)
            || !text(&w.command_draft, 2048)
            || w.log.len() > 128
        {
            return Err("Kernel work exceeds its supplied source/draft/log limits.".into());
        }
        if let Some(c) = &w.saved_config {
            c.validate()?;
        }
        w.outputs.validate_shape()?;
        w.store.validate()?;
        let mut records = Vec::new();
        if let Some(r) = &w.outputs.image {
            records.push(r);
        }
        if let Some(r) = &w.outputs.modules {
            records.push(r);
        }
        if let Some(i) = &w.outputs.initramfs {
            records.push(&i.image_build);
            if let Some(r) = &i.module_build {
                records.push(r);
            }
        }
        for f in &w.store.images {
            records.push(&f.build);
        }
        for f in &w.store.modules {
            records.push(&f.build);
        }
        for f in &w.store.initramfs {
            records.push(&f.image.image_build);
            if let Some(r) = &f.image.module_build {
                records.push(r);
            }
        }
        for record in records {
            self.record(record)?;
        }
        if let Some(path) = &w.saved_entry
            && (!boot_entry::path(path) || !w.store.entries.iter().any(|f| f.path == *path))
        {
            return Err("The saved-entry selection must name an actual virtual entry.".into());
        }
        if w.log
            .iter()
            .any(|l| !text(&l.input, 2048) || !text(&l.output, 4096))
        {
            return Err("Kernel console record exceeds the bounded text format.".into());
        }
        Ok(())
    }

    fn perform(&self, w: &mut Work, a: &Action) -> Result<String> {
        match a {
            Action::Source { index } => {
                if *index >= self.sources.len() {
                    return Err("Choose one supplied source.".into());
                }
                if w.source != *index {
                    w.source = *index;
                    w.checked = false;
                }
                Ok(
                    "Selected source changed; earlier build and staged outputs remain separate."
                        .into(),
                )
            }
            Action::Config { text: value } => {
                if !text(value, 8192) {
                    return Err("Keep the configuration draft within 8192 plain text bytes.".into());
                }
                if w.config_draft != *value {
                    w.config_draft = value.clone();
                    w.checked = false;
                }
                Ok("Configuration draft updated; save before building it.".into())
            }
            Action::SaveConfig => {
                let config = core::Config::parse(&w.config_draft)?;
                if w.saved_config.as_ref() != Some(&config) {
                    w.checked = false;
                }
                w.saved_config = Some(config);
                Ok("Saved the supported virtual configuration fragment.".into())
            }
            Action::CommandDraft { text: value } => {
                if !text(value, 2048) {
                    return Err("Keep the command draft within 2048 text bytes.".into());
                }
                w.command_draft = value.clone();
                Ok("Command draft retained; it has not run.".into())
            }
            Action::Run { input } => {
                let source = &self.sources[w.source];
                let before = (w.outputs.clone(), w.store.clone());
                let output = kernel_commands::execute(
                    kernel_commands::Context {
                        source_id: &source.id,
                        base_release: &source.release,
                        saved_config: w.saved_config.as_ref(),
                    },
                    &mut w.outputs,
                    &mut w.store,
                    input,
                )
                .map_err(|e| format!("{:?}: {}", e.kind, e.detail))?;
                if (w.outputs.clone(), w.store.clone()) != before {
                    w.checked = false;
                }
                w.command_draft.clear();
                w.line(input, &output, true);
                Ok(output)
            }
            Action::StageImage => {
                let image = w
                    .outputs
                    .image
                    .as_ref()
                    .ok_or("Build an image before staging it.")?;
                let path = w.store.image(&self.stage_root, image)?;
                w.checked = false;
                Ok(format!("Staged modeled image at {path}"))
            }
            Action::WorkingInitramfs => {
                w.outputs.assemble_initramfs()?;
                w.checked = false;
                Ok("Assembled modeled early userspace from the image and its required module inputs.".into())
            }
            Action::Initramfs { image_path } => {
                let path = w.store.assemble_initramfs(&self.stage_root, image_path)?;
                w.checked = false;
                Ok(format!(
                    "Created modeled initramfs at {path}; no host archive was generated."
                ))
            }
            Action::EntryDraft {
                filename,
                text: value,
            } => {
                if !text(filename, 48) || !text(value, 4096) {
                    return Err(
                        "Keep the entry name within 48 characters and its text within 4096 bytes."
                            .into(),
                    );
                }
                if w.entry_filename != *filename || w.entry_draft != *value {
                    w.entry_filename = filename.clone();
                    w.entry_draft = value.clone();
                    w.checked = false;
                }
                Ok("Boot-entry draft updated; save it to change the virtual file.".into())
            }
            Action::SaveEntry => {
                let path = w
                    .store
                    .entry(&self.stage_root, &w.entry_filename, &w.entry_draft)?;
                w.saved_entry = Some(path.clone());
                w.checked = false;
                Ok(format!("Saved virtual entry {path}"))
            }
            Action::Check => {
                w.checked = true;
                let results = self.checks(w);
                Ok(format!(
                    "{} of {} objective results pass",
                    results.iter().filter(|c| c.passed).count(),
                    results.len()
                ))
            }
            Action::Reset => {
                *w = Work::new(self);
                Ok("Only this task workspace has been reset.".into())
            }
        }
    }

    pub fn apply(&self, w: &mut Work, a: &Action) -> Result<String> {
        self.validate_work(w)?;
        let mut next = w.clone();
        match self.perform(&mut next, a) {
            Ok(output) => {
                self.validate_work(&next)?;
                *w = next;
                Ok(output)
            }
            Err(e) => {
                if let Action::Run { input } = a
                    && text(input, 2048)
                {
                    w.line(input, &e, false);
                }
                Err(e)
            }
        }
    }

    fn config_ready(&self, w: &Work) -> Result<core::Config> {
        let parsed = core::Config::parse(&w.config_draft)?;
        if w.saved_config.as_ref() != Some(&parsed) {
            return Err("Save the current configuration draft; the build uses the saved file, not unsaved editor text.".into());
        }
        if self.require_initrd_support && !parsed.initrd {
            return Err("This task requires external initramfs-loading support. Enable it in the saved configuration and rebuild affected outputs.".into());
        }
        Ok(parsed)
    }

    fn fallback_preserved(&self, w: &Work) -> bool {
        self.initial_store
            .images
            .iter()
            .all(|old| w.store.images.iter().find(|f| f.path == old.path) == Some(old))
            && self
                .initial_store
                .modules
                .iter()
                .all(|old| w.store.modules.iter().find(|f| f.path == old.path) == Some(old))
            && self
                .initial_store
                .initramfs
                .iter()
                .all(|old| w.store.initramfs.iter().find(|f| f.path == old.path) == Some(old))
            && self
                .initial_store
                .entries
                .iter()
                .all(|old| w.store.entries.iter().find(|f| f.path == old.path) == Some(old))
    }

    pub fn checks(&self, w: &Work) -> Vec<Check> {
        let check = |label: &str, result: Result<String>| match result {
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
        };
        if let Err(e) = self.validate_work(w) {
            return vec![check("Supplied workspace", Err(e))];
        }
        let config = self.config_ready(w);
        let Some(wanted) = self.sources.get(self.wanted_source) else {
            return vec![check(
                "Task source",
                Err("The task has no supplied target source.".into()),
            )];
        };
        let mut results = vec![check(
            "Saved configuration",
            config
                .as_ref()
                .map(|_| "Current configuration draft matches the saved supported file.".into())
                .map_err(Clone::clone),
        )];
        match self.kind {
            Kind::Image => {
                let image=match (&config,&w.outputs.image) {
                    (Ok(c),Some(image)) if image.current_for(&wanted.id,&wanted.release,c)=>Ok(format!("Image belongs to {} and the saved configuration.",image.release)),
                    (Err(e),_)=>Err(e.clone()),
                    _=>Err("Build the supplied target source with the current saved configuration; an earlier image does not update when its inputs change.".into()),
                };
                results.push(check("Current image build", image));
                let ready = w.outputs.readiness();
                results.push(check(
                    "Root-driver readiness",
                    if ready.passed {
                        Ok(ready.reason)
                    } else {
                        Err(ready.reason)
                    },
                ));
            }
            Kind::Boot => {
                let path = source_install::join(
                    &self.stage_root,
                    &format!("boot/loader/entries/{}", w.entry_filename),
                );
                let entry = path
                    .as_ref()
                    .ok()
                    .and_then(|p| w.store.entries.iter().find(|f| f.path == *p));
                let saved = entry.is_some_and(|f| {
                    Some(&f.path) == w.saved_entry.as_ref() && f.source == w.entry_draft
                });
                results.push(check(
                    "Saved independent entry",
                    if saved {
                        Ok("Current entry draft is saved in the virtual staging area.".into())
                    } else {
                        Err("Save the current entry filename and text before checking it.".into())
                    },
                ));
                let candidate = (|| -> Result<String> {
                    let entry =
                        entry.ok_or("Save a new candidate entry in the supplied staging area.")?;
                    if self
                        .initial_store
                        .entries
                        .iter()
                        .any(|old| old.path == entry.path)
                    {
                        return Err("Use a new entry filename; keep the supplied working fallback unchanged.".into());
                    }
                    let parsed =
                        boot_entry::Entry::parse(&entry.source).map_err(|e| e.to_string())?;
                    let image_path = format!(
                        "{}{}",
                        source_install::join(&self.stage_root, "boot")?,
                        parsed.linux
                    );
                    let image = w
                        .store
                        .images
                        .iter()
                        .find(|f| f.path == image_path)
                        .ok_or("The entry's linux path names no staged image.")?;
                    let c = config.as_ref().map_err(Clone::clone)?;
                    if !image.build.current_for(&wanted.id, &wanted.release, c) {
                        return Err("The selected staged image does not match the supplied target source and saved configuration. Stage the correct build and use its actual path.".into());
                    }
                    Ok(format!(
                        "New entry selects the staged target build {}.",
                        image.build.release
                    ))
                })();
                results.push(check("Staged target image", candidate));
                let boot = (|| -> Result<String> {
                    let path = path.map_err(|e| e.to_string())?;
                    let root = self
                        .root
                        .clone()
                        .ok_or("This task has no supplied root filesystem.")?;
                    let report = w.store.test_entry(&self.stage_root, &path, root)?;
                    if report.passed() {
                        Ok(format!(
                            "Shared boot model reaches the supplied root and init handoff with {}.",
                            report.release.unwrap_or_default()
                        ))
                    } else {
                        Err(report
                            .stages
                            .into_iter()
                            .find(|s| !s.passed)
                            .map(|s| s.detail)
                            .unwrap_or_else(|| {
                                "The supplied boot trial did not reach init handoff.".into()
                            }))
                    }
                })();
                results.push(check("Candidate boot trial", boot));
                results.push(check("Working fallback preserved",if self.fallback_preserved(w){Ok("The original virtual images, module trees and working entries remain unchanged.".into())}else{Err("A supplied fallback file or module tree changed. Restore its original build/entry and use a distinct version for the candidate.".into())}));
            }
        }
        results
    }

    pub fn solved(&self, w: &Work) -> bool {
        w.checked && self.checks(w).iter().all(|c| c.passed)
    }

    pub fn validate(&self) -> Result<()> {
        if !identity(&self.id)
            || self.instructions.trim().is_empty()
            || !text(&self.instructions, 1600)
            || self.sources.is_empty()
            || self.sources.len() > 2
            || self.initial_source >= self.sources.len()
            || self.wanted_source >= self.sources.len()
            || !source_install::directory(&self.stage_root)
            || self.reference.is_empty()
            || self.reference.len() > 24
            || !matches!(self.reference.last(), Some(Action::Check))
        {
            return Err("Supply a bounded named kernel task, sources, virtual root and reachable reference ending in Check.".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for source in &self.sources {
            if !identity(&source.id)
                || source.release.is_empty()
                || source.release.len() > 48
                || !identity(&source.release)
                || source.title.trim().is_empty()
                || !text(&source.title, 128)
                || !ids.insert(&source.id)
            {
                return Err(
                    "Kernel source descriptions need distinct bounded identities and releases."
                        .into(),
                );
            }
        }
        let mut w = Work::new(self);
        self.validate_work(&w)?;
        match self.kind {
            Kind::Image => {
                if self.root.is_some() || self.initial_store != kernel_stage::Store::default() {
                    return Err(
                        "An image-only task does not imply a staged boot environment.".into(),
                    );
                }
            }
            Kind::Boot => {
                let root = self
                    .root
                    .clone()
                    .ok_or("Boot staging needs its supplied root filesystem.")?;
                let required: std::collections::BTreeSet<_> =
                    root.required_drivers.iter().map(String::as_str).collect();
                if required
                    != std::collections::BTreeSet::from([
                        "nvme_core",
                        "nvme",
                        "jbd2",
                        "mbcache",
                        "ext4",
                    ])
                    || root.required_drivers.len() != 5
                {
                    return Err("This kernel-build fixture supports the declared NVMe/ext4 root and its supplied dependencies only.".into());
                }
                if self.initial_store.entries.is_empty() {
                    return Err(
                        "Supply a working fallback before asking for a staged handover.".into(),
                    );
                }
                for entry in &self.initial_store.entries {
                    if !self
                        .initial_store
                        .test_entry(&self.stage_root, &entry.path, root.clone())?
                        .passed()
                    {
                        return Err("The supplied preserved fallback must actually boot in the existing model.".into());
                    }
                }
            }
        }
        w.checked = true;
        if self.solved(&w) {
            return Err("The starter must leave meaningful outcomes unfinished.".into());
        }
        w.checked = false;
        for action in &self.reference {
            self.apply(&mut w, action)?;
        }
        if !self.solved(&w) {
            return Err(
                "The authored supported reference does not reach every actual outcome.".into(),
            );
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX_TASK_BYTES {
            return Err("Kernel task source exceeds its bounded persisted identity.".into());
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
            || !identity(&self.source_id)
            || self.assignment.id != self.source_id
            || self.example.id == self.assignment.id
        {
            return Err("Use a version 1 kernel definition with one assignment identity and a distinct worked example.".into());
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if self.assignment.kind != self.example.kind
            || (self.assignment.sources == self.example.sources
                && self.assignment.initial_config == self.example.initial_config
                && self.assignment.initial_store == self.example.initial_store)
        {
            return Err(
                "The worked example must teach the same activity with separate supplied inputs."
                    .into(),
            );
        }
        Ok(())
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
    let raw = serde_json::to_string(t).map_err(|e| e.to_string())?;
    if raw.len() > MAX_TASK_BYTES {
        return Err("The task's persisted source identity exceeds 65536 bytes.".into());
    }
    Ok(raw)
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
            return Err("Kernel work belongs to a different authored task revision; its original draft is retained.".into());
        }
        t.validate_work(&self.work)
    }
    pub fn validate_shape(&self) -> Result<()> {
        if self.source_task.len() > MAX_TASK_BYTES {
            return Err("Saved kernel source identity exceeds its byte limit.".into());
        }
        let task: Task = serde_json::from_str(&self.source_task)
            .map_err(|_| "Saved kernel work has malformed or unsupported task data.")?;
        task.validate()?;
        self.bind(&task)
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
        if self.version != 1 || d.version != 1 || self.source_id != d.source_id {
            return Err("Kernel session does not match the supplied scenario/version; original work retained.".into());
        }
        self.assignment.bind(&d.assignment)?;
        self.example.bind(&d.example)
    }
    pub fn validate_shape(&self) -> Result<()> {
        if self.version != 1 || !identity(&self.source_id) {
            return Err("Unknown kernel session identity/version.".into());
        }
        self.assignment.validate_shape()?;
        self.example.validate_shape()?;
        let assignment: Task =
            serde_json::from_str(&self.assignment.source_task).map_err(|e| e.to_string())?;
        let example: Task =
            serde_json::from_str(&self.example.source_task).map_err(|e| e.to_string())?;
        Definition {
            version: self.version,
            source_id: self.source_id.clone(),
            assignment,
            example,
        }
        .validate()
    }
    pub fn apply(&mut self, d: &Definition, practice: bool, a: &Action) -> Result<String> {
        self.bind(d)?;
        if practice {
            d.example.apply(&mut self.example.work, a)
        } else {
            d.assignment.apply(&mut self.assignment.work, a)
        }
    }
    pub fn current_solved(&self, d: &Definition) -> bool {
        self.bind(d).is_ok() && d.assignment.solved(&self.assignment.work)
    }
}
