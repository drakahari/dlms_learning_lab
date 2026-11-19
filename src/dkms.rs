//! Finite supplied DKMS lifecycle. No actual compiler, shell, module or filesystem.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_INPUT: usize = 2048;
pub const HELP: &str = "Simulated DKMS lifecycle: dkms add, build, install, uninstall or status. Use module/version or -m module -v version; one -k release selects a supplied kernel. Without -k, build/install/uninstall use the displayed running kernel. Build may add, install may add/build. Status shows added, built or installed, not loaded. Only supplied build-ready sources/kernels and one declared architecture are modeled. No real DKMS, compilation, shell, files, hooks, package actions or system changes. Paths, shell syntax, --force, --all, remove, unbuild, autoinstall and signing are outside this subset.";
type Result<T> = std::result::Result<T, String>;
fn token(s: &str, cap: usize) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && s.len() <= cap
        && !matches!(s, "." | "..")
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kernel {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub release: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub name: String,
    #[serde(deserialize_with = "crate::cron::text::<_,24>")]
    pub version: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub object: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Kernel,4>")]
    pub kernels: Vec<Kernel>,
    pub running: usize,
    #[serde(deserialize_with = "crate::cron::text::<_,16>")]
    pub architecture: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Source,8>")]
    pub sources: Vec<Source>,
}
impl Facts {
    pub fn validate(&self) -> Result<()> {
        if self.kernels.is_empty()
            || self.kernels.len() > 4
            || self.running >= self.kernels.len()
            || self.sources.is_empty()
            || self.sources.len() > 8
            || !matches!(self.architecture.as_str(), "x86_64" | "aarch64")
        {
            return Err("Supply one to four build-ready kernels, a running selection, one architecture and up to eight source versions.".into());
        }
        let mut kernels = BTreeSet::new();
        let mut sources = BTreeSet::new();
        for k in &self.kernels {
            if !token(&k.release, 64) || !kernels.insert(&k.release) {
                return Err(
                    "Kernel release labels must be distinct bounded literal values.".into(),
                );
            }
        }
        for s in &self.sources {
            if !token(&s.name, 32)
                || !token(&s.version, 24)
                || !token(&s.object, 32)
                || !sources.insert((&s.name, &s.version))
            {
                return Err(
                    "Supply distinct module/version pairs and bounded module object names.".into(),
                );
            }
            if self
                .sources
                .iter()
                .any(|other| other.name != s.name && other.object == s.object)
            {
                return Err("This single-object fixture excludes object-file collisions between different module packages.".into());
            }
        }
        Ok(())
    }
    pub fn source_path(&self, index: usize) -> Result<String> {
        let s = self.sources.get(index).ok_or("Choose a supplied source.")?;
        Ok(format!("/usr/src/{}-{}", s.name, s.version))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub source: usize,
    pub kernel: usize,
    pub installed: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,usize,8>")]
    pub registered: Vec<usize>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Record,32>")]
    pub records: Vec<Record>,
}
impl State {
    pub fn validate(&self, f: &Facts) -> Result<()> {
        f.validate()?;
        let registered: BTreeSet<_> = self.registered.iter().copied().collect();
        if self.registered.len() > 8
            || registered.len() != self.registered.len()
            || registered.iter().any(|i| *i >= f.sources.len())
            || self.records.len() > 32
        {
            return Err(
                "Saved DKMS registrations exceed or differ from the supplied source inventory."
                    .into(),
            );
        }
        let mut pairs = BTreeSet::new();
        let mut installed = BTreeSet::new();
        for r in &self.records {
            if !registered.contains(&r.source)
                || r.kernel >= f.kernels.len()
                || !pairs.insert((r.source, r.kernel))
            {
                return Err("Every built instance needs a registered supplied source/kernel pair, without duplicates.".into());
            }
            if r.installed && !installed.insert((&f.sources[r.source].name, r.kernel)) {
                return Err("Only one version of a module package can be installed for one supplied kernel.".into());
            }
        }
        Ok(())
    }
    pub fn installed(&self, f: &Facts, source: usize, kernel: usize) -> bool {
        self.validate(f).is_ok()
            && self
                .records
                .iter()
                .any(|r| r.source == source && r.kernel == kernel && r.installed)
    }
    pub fn virtual_files(&self, f: &Facts) -> Result<Vec<String>> {
        self.validate(f)?;
        let mut paths = self
            .records
            .iter()
            .filter(|r| r.installed)
            .map(|r| {
                format!(
                    "/lib/modules/{}/updates/dkms/{}.ko",
                    f.kernels[r.kernel].release, f.sources[r.source].object
                )
            })
            .collect::<Vec<_>>();
        paths.sort();
        Ok(paths)
    }
    fn register(&mut self, index: usize) {
        if !self.registered.contains(&index) {
            self.registered.push(index);
            self.registered.sort_unstable();
        }
    }
    fn build(&mut self, source: usize, kernel: usize) {
        self.register(source);
        if !self
            .records
            .iter()
            .any(|r| r.source == source && r.kernel == kernel)
        {
            self.records.push(Record {
                source,
                kernel,
                installed: false,
            });
            self.records.sort_by_key(|r| (r.source, r.kernel));
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Add,
    Build,
    Install,
    Uninstall,
    Status,
    Help,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub action: Action,
    pub module: Option<String>,
    pub version: Option<String>,
    pub kernel: Option<String>,
}
fn argument(slot: &mut Option<String>, value: &str, cap: usize) -> Result<()> {
    if slot.is_some() || !token(value, cap) {
        return Err("Use each option once with a bounded literal name, version or kernel release; paths and architecture suffixes are outside this subset.".into());
    }
    *slot = Some(value.into());
    Ok(())
}
pub fn parse(input: &str) -> Result<Command> {
    if input.len() > MAX_INPUT || !input.is_ascii() || input.chars().any(char::is_control) {
        return Err("Enter one supported DKMS command, using plain characters and at most 2,048 text bytes; command lists and control characters are not supported.".into());
    }
    if input.bytes().any(|b| b"'\"\\;$|&><`(){}[]*?#".contains(&b)) {
        return Err("Shell quoting, expansion, operators and recipes are outside this DKMS console. No operation was simulated.".into());
    }
    let words: Vec<_> = input.split_whitespace().collect();
    if words.len() < 2 || words.len() > 12 || words[0] != "dkms" {
        return Err("Use one listed dkms operation; this console is not a host shell.".into());
    }
    let action=match words[1]{"add"=>Action::Add,"build"=>Action::Build,"install"=>Action::Install,"uninstall"=>Action::Uninstall,"status"=>Action::Status,"-h"|"--help" if words.len()==2=>Action::Help,_=>return Err("That action/option is outside the supplied DKMS subset. Use dkms --help; no operation was simulated.".into())};
    let mut cmd = Command {
        action,
        module: None,
        version: None,
        kernel: None,
    };
    let mut at = 2;
    while at < words.len() {
        let word = words[at];
        at += 1;
        if ["-m", "-v", "-k"].contains(&word) {
            let value = words
                .get(at)
                .ok_or("The option needs a following literal value.")?;
            at += 1;
            match word {
                "-m" => argument(&mut cmd.module, value, 32)?,
                "-v" => argument(&mut cmd.version, value, 24)?,
                "-k" => argument(&mut cmd.kernel, value, 64)?,
                _ => unreachable!(),
            }
        } else if word.starts_with('-') {
            return Err("Only separate -m, -v and one -k are supported here; other options are not simulated.".into());
        } else if let Some((name, version)) = word.split_once('/') {
            argument(&mut cmd.module, name, 32)?;
            argument(&mut cmd.version, version, 24)?;
        } else {
            return Err("Use module/version, or -m module -v version. Bare paths and source archives are not simulated.".into());
        }
    }
    if cmd.version.is_some() && cmd.module.is_none() {
        return Err("A version filter also needs its module name (-m).".into());
    }
    if !matches!(action, Action::Status | Action::Help)
        && (cmd.module.is_none() || cmd.version.is_none())
    {
        return Err("This operation needs a module and version; inspect dkms status or use a supplied source description.".into());
    }
    if action == Action::Add && cmd.kernel.is_some() {
        return Err("Registration is source-wide; this initial subset does not accept -k with add. Use -k on build/install/uninstall/status.".into());
    }
    Ok(cmd)
}
pub fn status(f: &Facts, s: &State, cmd: &Command) -> Result<String> {
    s.validate(f)?;
    let mut lines = Vec::new();
    for &i in &s.registered {
        let source = &f.sources[i];
        if cmd.module.as_ref().is_some_and(|v| *v != source.name)
            || cmd.version.as_ref().is_some_and(|v| *v != source.version)
        {
            continue;
        }
        let records = s
            .records
            .iter()
            .filter(|r| r.source == i)
            .collect::<Vec<_>>();
        if records.is_empty() && cmd.kernel.is_none() {
            lines.push(format!("{}/{}: added", source.name, source.version));
        }
        for r in records {
            let kernel = &f.kernels[r.kernel].release;
            if cmd.kernel.as_ref().is_none_or(|v| v == kernel) {
                lines.push(format!(
                    "{}/{}, {}, {}: {}",
                    source.name,
                    source.version,
                    kernel,
                    f.architecture,
                    if r.installed { "installed" } else { "built" }
                ));
            }
        }
    }
    lines.sort();
    Ok(lines.join("\n"))
}
pub fn execute(f: &Facts, s: &mut State, input: &str) -> Result<String> {
    s.validate(f)?;
    let cmd = parse(input)?;
    if cmd.action == Action::Help {
        return Ok(HELP.into());
    }
    if cmd.action == Action::Status {
        return status(f, s, &cmd);
    }
    let source=f.sources.iter().position(|p|Some(&p.name)==cmd.module.as_ref()&&Some(&p.version)==cmd.version.as_ref()).ok_or("That source/version was not supplied. No source download, host-file lookup or build was attempted.")?;
    let kernel = if let Some(release) = &cmd.kernel {
        f.kernels.iter().position(|k|&k.release==release).ok_or("That kernel and its prepared build inputs are not supplied. No real build was attempted.")?
    } else {
        f.running
    };
    let mut next = s.clone();
    let label = format!("{}/{}", f.sources[source].name, f.sources[source].version);
    let output = match cmd.action {
        Action::Add => {
            if next.registered.contains(&source) {
                return Err("That source/version is already registered. Inspect dkms status; adding it again does not build or install it.".into());
            }
            next.register(source);
            format!("Registered supplied source {label}; no per-kernel build exists yet.")
        }
        Action::Build => {
            let existed = next
                .records
                .iter()
                .any(|r| r.source == source && r.kernel == kernel);
            next.build(source, kernel);
            format!(
                "{} {label} for {} ({}). This simulated build does not compile or test driver code.",
                if existed { "Already built" } else { "Built" },
                f.kernels[kernel].release,
                f.architecture
            )
        }
        Action::Install => {
            if next.records.iter().any(|r| {
                r.kernel == kernel
                    && r.installed
                    && r.source != source
                    && f.sources[r.source].name == f.sources[source].name
            }) {
                return Err("Automatic version replacement and module-version sanity checks are outside this subset. Explicitly uninstall the installed version for this kernel before installing its replacement; other kernels stay unchanged.".into());
            }
            let existed = next
                .records
                .iter()
                .any(|r| r.source == source && r.kernel == kernel && r.installed);
            next.build(source, kernel);
            for r in &mut next.records {
                if r.kernel == kernel && f.sources[r.source].name == f.sources[source].name {
                    r.installed = r.source == source;
                }
            }
            format!(
                "{} {label} for {}. Virtual placement only; installed does not mean loaded.",
                if existed {
                    "Already installed"
                } else {
                    "Installed"
                },
                f.kernels[kernel].release
            )
        }
        Action::Uninstall => {
            let r=next.records.iter_mut().find(|r|r.source==source&&r.kernel==kernel&&r.installed).ok_or("This subset models uninstall only for a currently installed instance. Inspect status first; absent/uninstalled invocation behavior is outside this subset and no operation was simulated.")?;
            r.installed = false;
            format!(
                "Uninstalled {label} for {}; its built record and source registration remain.",
                f.kernels[kernel].release
            )
        }
        Action::Help | Action::Status => unreachable!(),
    };
    next.validate(f)?;
    *s = next;
    Ok(output)
}
