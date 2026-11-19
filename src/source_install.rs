//! Bounded supplied-project builds and staged installations, entirely in memory.
//!
//! This is not a C compiler, an Autoconf shell runner or a general make interpreter.
//! A supplied source bundle has a typed program and fixed dependency graph. The
//! model derives artifacts and installation trees from that source/configuration;
//! no recipe, program bytes or path is passed to host tools.
use crate::{boot_entry, file_backup};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub mod command;

type Result<T> = std::result::Result<T, String>;
pub const MAX_FILES: usize = 192;

fn label(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}
fn name(s: &str) -> bool {
    label(s, 48)
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
        && !matches!(s, "." | "..")
}
pub fn directory(s: &str) -> bool {
    s == "/" || boot_entry::path(s)
}
pub fn join(root: &str, suffix: &str) -> Result<String> {
    if !directory(root)
        || suffix.is_empty()
        || suffix.starts_with('/')
        || !boot_entry::path(&format!("/{suffix}"))
    {
        return Err("Use a canonical virtual directory and a relative member without .. or empty components.".into());
    }
    let path = format!("{}/{suffix}", root.trim_end_matches('/'));
    if !boot_entry::path(&path) {
        return Err(
            "The combined virtual path exceeds the supported path format or length.".into(),
        );
    }
    Ok(path)
}

/// The supplied project's finite program semantics, not arbitrary learner code.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub name: String,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub version: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub default_greeting: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub archive: String,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub archive_root: String,
    pub source: file_backup::Tree,
    pub program: Program,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub default_prefix: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub manual: String,
    /// Text files to preserve outside the supplied source tree. They are virtual.
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,TextFile,16>")]
    pub live_files: Vec<TextFile>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextFile {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub text: String,
    pub mode: u16,
}
impl Facts {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || !boot_entry::path(&self.archive)
            || !self.archive.ends_with(".tar")
            || !name(&self.archive_root)
            || !self.source.valid()
            || self.source.entries.len() > 32
            || self
                .source
                .entries
                .values()
                .filter_map(|e| e.text.as_ref())
                .map(String::len)
                .sum::<usize>()
                > 32_768
            || !name(&self.program.name)
            || [
                "configure",
                "INSTALL",
                "README",
                "Makefile.in",
                "Makefile",
                "config.status",
            ]
            .contains(&self.program.name.as_str())
            || self.source.entries.contains_key(&self.program.name)
            || self.source.entries.contains_key("Makefile")
            || self.source.entries.contains_key("config.status")
            || !name(&self.program.version)
            || !label(&self.program.default_greeting, 64)
            || !self.program.default_greeting.is_ascii()
            || !directory(&self.default_prefix)
            || self.manual.len() > 4096
            || self.live_files.len() > 16
        {
            return Err(
                "Invalid supplied source project, archive, program or bounded file inventory."
                    .into(),
            );
        }
        for required in ["INSTALL", "configure", "Makefile.in", "README"] {
            if !self
                .source
                .entries
                .get(required)
                .is_some_and(|e| !e.node.directory && e.text.is_some())
            {
                return Err(format!(
                    "The supplied project needs a regular {required} member."
                ));
            }
        }
        if self.source.entries["README"].text.as_deref() != Some(self.manual.as_str())
            || self.source.entries["configure"].node.mode & 0o100 == 0
            || self.source.entries.values().any(|e| e.node.mode > 0o777)
        {
            return Err("Supply the approved README text, an executable configure entry point and ordinary source modes.".into());
        }
        let defaults = Configuration {
            prefix: self.default_prefix.clone(),
            greeting: self.program.default_greeting.clone(),
            uppercase: false,
        };
        World::installed_paths(self, &defaults, None)?;
        let mut paths = std::collections::BTreeSet::new();
        for f in &self.live_files {
            if !boot_entry::path(&f.path)
                || !paths.insert(&f.path)
                || f.path == self.archive
                || f.text.len() > 4096
                || f.mode > 0o777
            {
                return Err(
                    "Supply bounded unique live text paths and ordinary permission modes.".into(),
                );
            }
        }
        for relative in self.source.entries.keys().filter(|p| !p.is_empty()) {
            // Existing archive validation rejects links/traversal; this project
            // additionally restricts members to the disclosed literal path form.
            join("/", relative)?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub prefix: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub greeting: String,
    pub uppercase: bool,
}
impl Configuration {
    pub fn validate(&self) -> Result<()> {
        if !directory(&self.prefix) || !label(&self.greeting, 64) || !self.greeting.is_ascii() {
            return Err(
                "Use an absolute virtual prefix and 1–64 ordinary ASCII greeting characters."
                    .into(),
            );
        }
        // Ensure every derived install destination fits before changing state.
        join(&self.prefix, "share/doc/supplied-project/README")?;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Artifact {
    pub program: Program,
    pub configuration: Configuration,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}
impl Output {
    fn success(stdout: String) -> Self {
        Self {
            stdout,
            stderr: String::new(),
            status: 0,
        }
    }
}
impl Artifact {
    pub fn run(&self, args: &[String]) -> Result<Output> {
        if args.len() > 4
            || args
                .iter()
                .any(|a| a.len() > 128 || a.chars().any(char::is_control))
        {
            return Err(
                "Keep the supplied program invocation within four short literal arguments.".into(),
            );
        }
        match args
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["--version"] => Ok(Output::success(format!(
                "{} {}\n",
                self.program.name, self.program.version
            ))),
            ["--prefix"] => Ok(Output::success(format!("{}\n", self.configuration.prefix))),
            [person] if !person.starts_with('-') => {
                let text = format!("{} {}\n", self.configuration.greeting, person);
                Ok(Output::success(if self.configuration.uppercase {
                    text.to_ascii_uppercase()
                } else {
                    text
                }))
            }
            _ => Ok(Output {
                stdout: String::new(),
                stderr: format!("Usage: {} NAME | --version | --prefix\n", self.program.name),
                status: 2,
            }),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum File {
    Archive,
    Text { text: String, mode: u16 },
    Program { artifact: Artifact, mode: u16 },
}
impl File {
    pub fn mode(&self) -> u16 {
        match self {
            Self::Archive => 0o444,
            Self::Text { mode, .. } | Self::Program { mode, .. } => *mode,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
    pub directories: BTreeSet<String>,
    pub files: BTreeMap<String, File>,
    pub source_root: Option<String>,
    pub configuration: Option<Configuration>,
    pub built: Option<Artifact>,
    /// Exact artifact checked; reconfiguration makes this observation stale.
    pub checked: Option<Artifact>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    All,
    Check,
    Install,
    Clean,
}
fn validate_files(files: &BTreeMap<String, File>) -> Result<()> {
    if files.len() > MAX_FILES {
        return Err("Virtual file inventory limit reached.".into());
    }
    for (path, file) in files {
        if !boot_entry::path(path) {
            return Err("Invalid virtual file path.".into());
        }
        let mut ancestor = path.as_str();
        while let Some((parent, _)) = ancestor.rsplit_once('/') {
            if files.contains_key(parent) {
                return Err("A virtual regular file cannot also be a parent directory; select a different destination.".into());
            }
            ancestor = parent;
        }
        if file.mode() > 0o777 || matches!(file, File::Text { text, .. } if text.len() > 32_768) {
            return Err("Virtual file contents or permissions exceed the supplied subset.".into());
        }
    }
    Ok(())
}
fn parent_directories(path: &str, directories: &mut BTreeSet<String>) {
    let mut at = path;
    while let Some((parent, _)) = at.rsplit_once('/') {
        directories.insert(if parent.is_empty() { "/" } else { parent }.into());
        at = parent;
    }
}
fn layout(files: &BTreeMap<String, File>, directories: &BTreeSet<String>) -> Result<()> {
    validate_files(files)?;
    if directories.len() > 384
        || directories
            .iter()
            .any(|d| !directory(d) || files.contains_key(d))
    {
        return Err("A virtual directory cannot also be a regular file; choose another destination or reset this attempt.".into());
    }
    Ok(())
}
impl World {
    pub fn new(facts: &Facts) -> Result<Self> {
        facts.validate()?;
        let mut files: BTreeMap<String, File> = facts
            .live_files
            .iter()
            .map(|f| {
                (
                    f.path.clone(),
                    File::Text {
                        text: f.text.clone(),
                        mode: f.mode,
                    },
                )
            })
            .collect();
        files.insert(facts.archive.clone(), File::Archive);
        let mut directories = BTreeSet::from(["/".into()]);
        for p in files.keys() {
            parent_directories(p, &mut directories);
        }
        layout(&files, &directories)?;
        Ok(Self {
            directories,
            files,
            source_root: None,
            configuration: None,
            built: None,
            checked: None,
        })
    }
    pub fn mkdir(&mut self, path: &str, parents: bool) -> Result<()> {
        if !directory(path) {
            return Err("Use a canonical absolute virtual directory.".into());
        }
        if self.directories.contains(path) {
            return if parents {
                Ok(())
            } else {
                Err("That virtual directory already exists.".into())
            };
        }
        let parent = path
            .rsplit_once('/')
            .map(|(p, _)| if p.is_empty() { "/" } else { p })
            .unwrap_or("/");
        if !parents && !self.directories.contains(parent) {
            return Err("The parent directory is missing; use mkdir -p or create it first.".into());
        }
        let mut next = self.directories.clone();
        parent_directories(path, &mut next);
        next.insert(path.into());
        layout(&self.files, &next)?;
        self.directories = next;
        Ok(())
    }
    pub fn extract(&mut self, facts: &Facts, destination: &str) -> Result<()> {
        facts.validate()?;
        if self.files.get(&facts.archive) != Some(&File::Archive) {
            return Err(
                "The supplied virtual archive is no longer present at its recorded path.".into(),
            );
        }
        let root = join(destination, &facts.archive_root)?;
        if facts.archive == root || facts.archive.starts_with(&format!("{root}/")) {
            return Err("The source directory would overlap the supplied archive file; choose another destination.".into());
        }
        let mut files = self.files.clone();
        let mut directories = self.directories.clone();
        parent_directories(&root, &mut directories);
        directories.insert(root.clone());
        for (member, entry) in &facts.source.entries {
            if entry.node.directory && !member.is_empty() {
                let path = join(&root, member)?;
                parent_directories(&path, &mut directories);
                directories.insert(path);
            }
            if let Some(text) = &entry.text {
                let path = join(&root, member)?;
                if self
                    .files
                    .get(&path)
                    .is_some_and(|f| !matches!(f, File::Text { .. }))
                {
                    return Err("Extraction would replace a virtual program with source text; select another source directory.".into());
                }
                files.insert(
                    path,
                    File::Text {
                        text: text.clone(),
                        mode: entry.node.mode,
                    },
                );
            }
        }
        for p in files.keys() {
            parent_directories(p, &mut directories);
        }
        layout(&files, &directories)?;
        // Selecting a different extraction does not transplant another build.
        if self.source_root.as_ref() != Some(&root) {
            self.configuration = None;
            self.built = None;
            self.checked = None;
        }
        self.files = files;
        self.directories = directories;
        self.source_root = Some(root);
        Ok(())
    }
    pub fn configure(&mut self, facts: &Facts, configuration: Configuration) -> Result<()> {
        facts.validate()?;
        configuration.validate()?;
        let root = self
            .source_root
            .as_ref()
            .ok_or("Unpack the supplied source archive first.")?;
        for member in ["configure", "Makefile.in", "INSTALL"] {
            let actual = self.files.get(&join(root, member)?);
            let expected = &facts.source.entries[member];
            if actual
                != expected
                    .text
                    .as_ref()
                    .map(|text| File::Text {
                        text: text.clone(),
                        mode: expected.node.mode,
                    })
                    .as_ref()
            {
                return Err("The selected source tree no longer contains the supplied project inputs; unpack again.".into());
            }
        }
        // Existing installed artifacts remain what they were. Do not silently
        // rewrite a staging tree when configuration changes.
        let mut files = self.files.clone();
        let status = format!(
            "prefix={}\ngreeting={}\nuppercase={}\n",
            configuration.prefix, configuration.greeting, configuration.uppercase
        );
        files.insert(
            join(root, "config.status")?,
            File::Text {
                text: status.clone(),
                mode: 0o644,
            },
        );
        files.insert(join(root, "Makefile")?, File::Text {
            text: format!("# Supplied simulated dependency graph; not arbitrary recipe execution\n{status}all: {}\ncheck: all\ninstall: all\nclean:\n", facts.program.name), mode: 0o644 });
        layout(&files, &self.directories)?;
        self.files = files;
        self.configuration = Some(configuration);
        self.checked = None;
        Ok(())
    }
    pub fn current_artifact(&self, facts: &Facts) -> Result<Artifact> {
        Ok(Artifact {
            program: facts.program.clone(),
            configuration: self
                .configuration
                .clone()
                .ok_or("Run the supplied configure entry point before make.")?,
        })
    }
    pub fn is_build_current(&self, facts: &Facts) -> bool {
        self.current_artifact(facts).ok().as_ref().is_some_and(|a| {
            self.built.as_ref() == Some(a)
                && self
                    .binary_member(facts)
                    .ok()
                    .and_then(|p| self.files.get(&p))
                    .is_some_and(
                        |f| matches!(f, File::Program { artifact, mode: 0o755 } if artifact == a),
                    )
        })
    }
    pub fn is_check_current(&self, facts: &Facts) -> bool {
        self.is_build_current(facts) && self.checked == self.built
    }
    pub fn binary_member(&self, facts: &Facts) -> Result<String> {
        join(
            self.source_root
                .as_deref()
                .ok_or("Unpack the supplied archive first.")?,
            &facts.program.name,
        )
    }
    pub fn installed_paths(
        facts: &Facts,
        config: &Configuration,
        destdir: Option<&str>,
    ) -> Result<(String, String)> {
        config.validate()?;
        let prefix = match destdir {
            None => config.prefix.clone(),
            Some(root) => {
                if !directory(root) {
                    return Err("DESTDIR must be an absolute canonical virtual directory.".into());
                }
                if config.prefix == "/" {
                    root.into()
                } else {
                    join(root, &config.prefix[1..])?
                }
            }
        };
        Ok((
            join(&prefix, &format!("bin/{}", facts.program.name))?,
            join(&prefix, &format!("share/doc/{}/README", facts.program.name))?,
        ))
    }
    /// Supplied fixed graph: check -> all; install -> all. No recipe text runs.
    pub fn make(&mut self, facts: &Facts, target: Target, destdir: Option<&str>) -> Result<Output> {
        facts.validate()?;
        if destdir.is_some() && target != Target::Install {
            return Err("DESTDIR is supported only for this project's install target.".into());
        }
        let root = self
            .source_root
            .as_ref()
            .ok_or("Unpack the supplied project first.")?;
        if self.configuration.is_none() {
            return Err(
                "No generated Makefile: run ./configure in the selected source directory first."
                    .into(),
            );
        }
        let binary = join(root, &facts.program.name)?;
        if target == Target::Clean {
            self.files.remove(&binary);
            self.built = None;
            self.checked = None;
            return Ok(Output::success(
                "Removed supplied build artifacts; installed copies are unchanged.\n".into(),
            ));
        }
        let artifact = self.current_artifact(facts)?;
        let mut files = self.files.clone();
        let mut out = String::new();
        if self.built.as_ref() != Some(&artifact)
            || !matches!(files.get(&binary), Some(File::Program { artifact: a, mode: 0o755 }) if a == &artifact)
        {
            files.insert(
                binary,
                File::Program {
                    artifact: artifact.clone(),
                    mode: 0o755,
                },
            );
            out.push_str(&format!(
                "Building {} from supplied source and current configuration\n",
                facts.program.name
            ));
        } else {
            out.push_str("Build target is up to date.\n");
        }
        let mut checked = self.checked.clone();
        if target == Target::Check {
            // Invoke real typed program behavior for several deterministic cases.
            for person in ["Ada", "Rio"] {
                let actual = artifact.run(&[person.into()])?;
                let expected = format!("{} {person}\n", artifact.configuration.greeting);
                let expected = if artifact.configuration.uppercase {
                    expected.to_ascii_uppercase()
                } else {
                    expected
                };
                if actual.status != 0 || actual.stdout != expected || !actual.stderr.is_empty() {
                    return Err(format!("Supplied program self-check failed for {person}."));
                }
            }
            checked = Some(artifact.clone());
            out.push_str("PASS: supplied program self-checks (2 cases).\n");
        }
        if target == Target::Install {
            let (program, manual) = Self::installed_paths(facts, &artifact.configuration, destdir)?;
            files.insert(
                program.clone(),
                File::Program {
                    artifact: artifact.clone(),
                    mode: 0o755,
                },
            );
            let source_readme = join(root, "README")?;
            let Some(File::Text { text, .. }) = self.files.get(&source_readme) else {
                return Err(
                    "The supplied source README is missing; restore it before installation.".into(),
                );
            };
            files.insert(
                manual.clone(),
                File::Text {
                    text: text.clone(),
                    mode: 0o644,
                },
            );
            out.push_str(&format!("Installed virtual files:\n{program}\n{manual}\n"));
        }
        let mut directories = self.directories.clone();
        for p in files.keys() {
            parent_directories(p, &mut directories);
        }
        layout(&files, &directories)?;
        self.files = files;
        self.directories = directories;
        self.built = Some(artifact);
        self.checked = checked;
        Ok(Output::success(out))
    }
    pub fn run(&self, path: &str, args: &[String]) -> Result<Output> {
        if !boot_entry::path(path) {
            return Err("Use a supplied canonical virtual program path.".into());
        }
        match self.files.get(path) {
            Some(File::Program { artifact, mode }) if mode & 0o111 != 0 => artifact.run(args),
            Some(_) => Err("That virtual file is not an executable supplied program.".into()),
            None => Err(
                "No program is installed at that virtual path; inspect the staged destinations."
                    .into(),
            ),
        }
    }
}
