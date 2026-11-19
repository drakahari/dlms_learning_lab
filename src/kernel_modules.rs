//! Finite loadable-module laboratory. No kernel calls, ELF loading or host paths.
//! The catalogue is caller-held; runtime state cannot introduce module metadata.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CONFIG_PATH: &str = "/etc/modprobe.d/lab.conf";
pub const MAX_COMMAND: usize = 4096;
pub const MAX_CONFIG: usize = 8192;
pub const HELP: &str = "Simulated supplied loadable modules only: uname -r; lspci -k; lsmod; modinfo NAME or supplied absolute object path; modprobe NAME [INTEGER_PARAMETER=VALUE ...]; modprobe -r NAME; insmod SUPPLIED_OBJECT_PATH [INTEGER_PARAMETER=VALUE ...]; rmmod NAME; depmod -a; dmesg; cat /etc/modprobe.d/lab.conf; cat /sys/module/NAME/parameters/PARAMETER. Edit the supplied options file in the native editor. Only options NAME key=value lines, full-line comments and blank lines are supported. No aliases, built-in drivers, blacklist, install/remove scripts, force, ELF/kernel execution, automatic device loading, shell expansion, pipelines or host access. File paths are virtual lookup keys. Duplicate scalar assignments and unknown parameter names are explicitly unsupported. Integers use decimal, or octal with a leading zero; hex is outside this subset.";

/// Additional exact-name capability is opt-in; all preceding consoles retain HELP.
pub const NAMED_HELP: &str = "Virtual named-load console: uname -r; lspci -k; lsmod; modinfo PHYSICAL_NAME or supplied object path; modprobe PHYSICAL_NAME_OR_EXACT_ALIAS [integer parameters]; modprobe -r PHYSICAL_NAME; insmod SUPPLIED_OBJECT_PATH [integer parameters]; rmmod PHYSICAL_NAME; depmod -a; dmesg; cat the supplied options file or effective parameter. The single virtual options file accepts options PHYSICAL_NAME key=integer and alias EXACT_NAME PHYSICAL_NAME. Only eight unique literal aliases; no wildcard, chains, collisions with physical names, duplicate alias lines, alias-specific options, removal by alias, install/remove scripts, blacklist, force or host access. Saving affects later requests, not an existing instance. Unknown syntax is refused honestly.";

fn name(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
/// Canonical identifier used by literal module and exact alias names.
pub fn normalized_name(s: &str) -> Result<String, String> {
    module_name(s)
}
fn module_name(s: &str) -> Result<String, String> {
    let s = s.replace('-', "_");
    if name(&s) {
        Ok(s)
    } else {
        Err("Use one supplied module name, not a path or option.".into())
    }
}
fn plain(s: &str, cap: usize) -> bool {
    !s.is_empty() && s.len() <= cap && !s.chars().any(char::is_control)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    pub description: String,
    pub minimum: i64,
    pub maximum: i64,
    pub default: i64,
}
impl Parameter {
    fn contains(&self, n: i64) -> bool {
        (self.minimum..=self.maximum).contains(&n)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub path: String,
    pub description: String,
    pub kernel_release: String,
    pub size: u32,
    pub dependencies: Vec<String>,
    pub parameters: BTreeMap<String, Parameter>,
    /// Fixed external references in this exercise; not a real process refcount.
    pub external_users: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub slot: String,
    pub description: String,
    pub driver: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalogue {
    pub kernel_release: String,
    pub modules: BTreeMap<String, Module>,
    pub devices: Vec<Device>,
}
impl Catalogue {
    pub fn validate(&self) -> Result<(), String> {
        if !plain(&self.kernel_release, 64)
            || !self
                .kernel_release
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            || self.modules.is_empty()
            || self.modules.len() > 32
            || self.devices.len() > 16
        {
            return Err("Module catalogue exceeds the finite supplied-system limits.".into());
        }
        let mut paths = BTreeSet::new();
        for (n, m) in &self.modules {
            let root = format!("/lib/modules/{}/", m.kernel_release);
            let relative = m
                .path
                .strip_prefix(&root)
                .ok_or("Object path must belong to its declared virtual kernel release.")?;
            if !name(n)
                || !plain(&m.description, 240)
                || !plain(&m.kernel_release, 64)
                || !m
                    .kernel_release
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                || m.path.len() > 512
                || !m.path.ends_with(".ko")
                || relative.split('/').any(|p| {
                    p.is_empty()
                        || p == "."
                        || p == ".."
                        || !p
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                })
                || !paths.insert(&m.path)
                || m.dependencies.len() > 8
                || m.parameters.len() > 8
                || m.external_users > 1000
                || m.dependencies.iter().collect::<BTreeSet<_>>().len() != m.dependencies.len()
                || m.dependencies.iter().any(|d| !self.modules.contains_key(d))
                || m.parameters.iter().any(|(p, s)| {
                    !name(p)
                        || !plain(&s.description, 160)
                        || s.minimum > s.maximum
                        || !s.contains(s.default)
                })
            {
                return Err(
                    "Invalid supplied module metadata, dependency, parameter or object path."
                        .into(),
                );
            }
        }
        fn visit(
            n: &str,
            c: &Catalogue,
            active: &mut BTreeSet<String>,
            done: &mut BTreeSet<String>,
        ) -> Result<(), String> {
            if done.contains(n) {
                return Ok(());
            }
            if !active.insert(n.to_owned()) {
                return Err(
                    "Cyclic hard dependencies are not a valid supplied module catalogue.".into(),
                );
            }
            for d in &c.modules[n].dependencies {
                visit(d, c, active, done)?;
            }
            active.remove(n);
            done.insert(n.into());
            Ok(())
        }
        let mut done = BTreeSet::new();
        for n in self.modules.keys() {
            visit(n, self, &mut BTreeSet::new(), &mut done)?;
        }
        let mut slots = BTreeSet::new();
        for d in &self.devices {
            if !plain(&d.slot, 32)
                || !plain(&d.description, 240)
                || !slots.insert(&d.slot)
                || !self.modules.contains_key(&d.driver)
            {
                return Err(
                    "Supplied devices need unique slots and known loadable drivers.".into(),
                );
            }
        }
        Ok(())
    }
    pub fn object(&self, path: &str) -> Option<&str> {
        self.modules
            .iter()
            .find_map(|(n, m)| (m.path == path).then_some(n.as_str()))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Loaded {
    pub parameters: BTreeMap<String, i64>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub index: BTreeSet<String>,
    pub loaded: BTreeMap<String, Loaded>,
    pub config: String,
    pub diagnostics: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub status: u8,
    pub stdout: String,
    pub stderr: String,
}
impl Outcome {
    fn ok(stdout: impl Into<String>) -> Self {
        Self {
            status: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }
    fn failed(stderr: impl Into<String>) -> Self {
        Self {
            status: 1,
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }
    pub fn display(&self) -> String {
        if self.status == 0 {
            self.stdout.clone()
        } else {
            format!(
                "{}\n[Command failed · exit status {}]\n",
                self.stderr.trim_end(),
                self.status
            )
        }
    }
}
type Options = BTreeMap<String, String>;
/// Linux-style integer subset: decimal or leading-zero octal, optional minus.
/// Hex/plus/other syntax is refused by the literal assignment grammar.
pub fn integer(text: &str) -> Option<i64> {
    let body = text.strip_prefix('-').unwrap_or(text);
    if body.is_empty() || !body.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    i64::from_str_radix(
        text,
        if body.len() > 1 && body.starts_with('0') {
            8
        } else {
            10
        },
    )
    .ok()
}
fn known_options(c: &Catalogue, n: &str, options: &Options) -> Result<(), String> {
    if let Some(k) = options
        .keys()
        .find(|k| !c.modules[n].parameters.contains_key(*k))
    {
        return Err(format!(
            "Unsupported parameter {k} for supplied module {n}. Use modinfo to inspect the supported names. Real kernels may ignore unknown parameters; that behavior is outside this subset."
        ));
    }
    Ok(())
}

fn assignments(words: &[String]) -> Result<Options, String> {
    if words.len() > 8 {
        return Err("At most eight scalar parameters are supported.".into());
    }
    let mut values = BTreeMap::new();
    for w in words {
        let (key, value) = w
            .split_once('=')
            .ok_or("Parameters use NAME=INTEGER, for example queues=2.")?;
        if !name(key)
            || value.len() > 21
            || value.is_empty()
            || !value
                .strip_prefix('-')
                .unwrap_or(value)
                .bytes()
                .all(|b| b.is_ascii_digit())
        {
            return Err(
                "This exercise supports integer digits with an optional minus; a leading zero selects octal. Hex and other parameter syntax are unsupported.".into(),
            );
        }
        if values.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err("Duplicate scalar assignments are outside this module subset; keep one value for each parameter.".into());
        }
    }
    Ok(values)
}
/// Configuration is data, not a shell. Invalid drafts can be saved for diagnosis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Configuration {
    pub options: BTreeMap<String, Options>,
    pub aliases: BTreeMap<String, String>,
}
/// Existing physical-name consoles deliberately continue to reject alias directives.
pub fn parse_config(text: &str, c: &Catalogue) -> Result<BTreeMap<String, Options>, String> {
    configuration(text, c, false).map(|c| c.options)
}
/// Exact literal aliases only; never wildcard matching, chaining or executable text.
pub fn parse_named_config(text: &str, c: &Catalogue) -> Result<Configuration, String> {
    configuration(text, c, true)
}
fn configuration(text: &str, c: &Catalogue, named: bool) -> Result<Configuration, String> {
    if text.len() > MAX_CONFIG || text.lines().count() > 64 || text.contains('\0') {
        return Err("Module options file exceeds 8 KiB/64 lines or contains a NUL.".into());
    }
    let mut result: BTreeMap<String, Options> = BTreeMap::new();
    let mut aliases = BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let w: Vec<String> = line.split_whitespace().map(str::to_owned).collect();
        if named && w.first().is_some_and(|w| w == "alias") {
            if w.len() != 3 {
                return Err(format!(
                    "Line {}: use alias EXACT_NAME PHYSICAL_MODULE; no flags, globs or extra operands.",
                    i + 1
                ));
            }
            let alias = module_name(&w[1])?;
            let physical = module_name(&w[2])?;
            if c.modules.contains_key(&alias) {
                return Err(format!(
                    "Line {}: alias collisions with supplied physical names are outside this subset.",
                    i + 1
                ));
            }
            if !c.modules.contains_key(&physical) {
                return Err(format!(
                    "Line {}: an alias must point directly to a supplied physical module; chains and missing targets are unsupported.",
                    i + 1
                ));
            }
            if aliases.insert(alias, physical).is_some() {
                return Err(format!(
                    "Line {}: duplicate alias names are unsupported; keep one unambiguous binding.",
                    i + 1
                ));
            }
            if aliases.len() > 8 {
                return Err("At most eight literal aliases are supported.".into());
            }
            continue;
        }
        if w.len() < 3 || w[0] != "options" {
            return Err(format!(
                "Line {}: supported configuration is options MODULE parameter=integer. Other directives/continuations are unsupported and never executed.",
                i + 1
            ));
        }
        let n = module_name(&w[1])?;
        if !c.modules.contains_key(&n) {
            if named {
                return Err(format!(
                    "Line {}: options require a supplied physical module name; alias-specific options are unsupported.",
                    i + 1
                ));
            }
            return Err(format!("Line {}: no supplied module named {n}.", i + 1));
        }
        let opts = assignments(&w[2..])?;
        let dst = result.entry(n).or_default();
        for (k, v) in opts {
            if dst.insert(k, v).is_some() {
                return Err(format!(
                    "Line {}: duplicate scalar options are unsupported; keep one assignment.",
                    i + 1
                ));
            }
        }
    }
    Ok(Configuration {
        options: result,
        aliases,
    })
}
impl Runtime {
    pub fn validate(&self, c: &Catalogue) -> Result<(), String> {
        c.validate()?;
        if self.index.len() > c.modules.len()
            || self.index.iter().any(|n| !c.modules.contains_key(n))
            || self.loaded.len() > c.modules.len()
            || self.config.len() > MAX_CONFIG
            || self.config.lines().count() > 64
            || self.config.contains('\0')
            || self.diagnostics.len() > 32
            || self.diagnostics.iter().any(|d| !plain(d, 512))
        {
            return Err("Invalid or oversized virtual module runtime.".into());
        }
        for (n, loaded) in &self.loaded {
            let m = c
                .modules
                .get(n)
                .ok_or("Loaded module is not in the supplied catalogue.")?;
            if m.kernel_release != c.kernel_release
                || m.dependencies.iter().any(|d| !self.loaded.contains_key(d))
                || loaded.parameters.len() != m.parameters.len()
                || loaded
                    .parameters
                    .iter()
                    .any(|(k, v)| m.parameters.get(k).is_none_or(|p| !p.contains(*v)))
            {
                return Err("Loaded modules need compatible releases, all dependencies and valid effective parameters.".into());
            }
        }
        Ok(())
    }
    fn log(&mut self, message: String) {
        // Only bounded catalogue identifiers/fixed diagnostics reach this buffer.
        self.diagnostics.push(message);
        if self.diagnostics.len() > 32 {
            self.diagnostics.remove(0);
        }
    }
    pub fn users(&self, c: &Catalogue, n: &str) -> Vec<String> {
        self.loaded
            .keys()
            .filter(|other| c.modules[*other].dependencies.iter().any(|d| d == n))
            .cloned()
            .collect()
    }
    fn insert(
        &mut self,
        c: &Catalogue,
        n: &str,
        options: &Options,
        present: Option<&BTreeSet<String>>,
    ) -> Result<(), String> {
        let m = &c.modules[n];
        if present.is_some_and(|objects| !objects.contains(n)) {
            return Err(format!(
                "{}: No such file or directory in this simulated image.",
                m.path
            ));
        }
        if self.loaded.contains_key(n) {
            return Err(format!("{n}: File exists (already loaded)"));
        }
        if m.kernel_release != c.kernel_release {
            self.log(format!(
                "{n}: version magic {} does not match {}",
                m.kernel_release, c.kernel_release
            ));
            return Err(format!(
                "{n}: Invalid module format; inspect dmesg and uname -r."
            ));
        }
        let missing: Vec<_> = m
            .dependencies
            .iter()
            .filter(|d| !self.loaded.contains_key(*d))
            .collect();
        if !missing.is_empty() {
            self.log(format!(
                "{n}: unresolved dependency; load its supplied prerequisites first"
            ));
            return Err(format!(
                "{n}: Unknown symbol in module; inspect modinfo and dmesg."
            ));
        }
        let mut params: BTreeMap<_, _> = m
            .parameters
            .iter()
            .map(|(k, p)| (k.clone(), p.default))
            .collect();
        for (k, v) in options {
            let number = integer(v);
            if number.is_none()
                || m.parameters
                    .get(k)
                    .is_none_or(|p| !p.contains(number.unwrap_or(0)))
            {
                self.log(format!(
                    "{n}: invalid parameter {k}; inspect modinfo for the supported name and range"
                ));
                return Err(format!("{n}: Invalid argument; inspect dmesg and modinfo."));
            }
            params.insert(k.clone(), number.expect("checked integer"));
        }
        self.loaded
            .insert(n.to_owned(), Loaded { parameters: params });
        Ok(())
    }
    fn load(
        &mut self,
        c: &Catalogue,
        n: &str,
        saved: &BTreeMap<String, Options>,
        extra: &Options,
        present: Option<&BTreeSet<String>>,
    ) -> Result<(), String> {
        if self.loaded.contains_key(n) {
            return Ok(());
        }
        if !self.index.contains(n) {
            return Err(format!(
                "Module {n} not found in the current virtual index for {}. Inspect the supplied object and rebuild with depmod -a if appropriate.",
                c.kernel_release
            ));
        }
        for dep in &c.modules[n].dependencies {
            self.load(c, dep, saved, &Options::new(), present)?;
        }
        let mut options = saved.get(n).cloned().unwrap_or_default();
        for (k, v) in extra {
            if options.insert(k.clone(), v.clone()).is_some() {
                return Err("Duplicate saved/command-line scalar options are outside this subset; keep a single source for each value.".into());
            }
        }
        self.insert(c, n, &options, present)
    }
    fn remove(&mut self, c: &Catalogue, n: &str, dependencies: bool) -> Result<(), String> {
        if !self.loaded.contains_key(n) {
            return if dependencies {
                Ok(())
            } else {
                Err(format!("Module {n} is not currently loaded."))
            };
        }
        if !self.users(c, n).is_empty() || c.modules[n].external_users > 0 {
            return Err(format!(
                "Module {n} is in use; inspect lsmod. This exercise has no force removal."
            ));
        }
        self.loaded.remove(n);
        if dependencies {
            for dep in &c.modules[n].dependencies {
                if self.loaded.contains_key(dep)
                    && self.users(c, dep).is_empty()
                    && c.modules[dep].external_users == 0
                {
                    self.remove(c, dep, true)?;
                }
            }
        }
        Ok(())
    }
    /// One bounded literal command. Operational failure may retain already loaded
    /// dependencies/diagnostics; unsupported syntax makes no changes.
    pub fn execute(&mut self, c: &Catalogue, input: &str) -> Result<Outcome, String> {
        self.execute_inner(c, input, false, None)
    }
    /// Only a caller-held activity may opt into exact configured load names.
    pub fn execute_named(&mut self, c: &Catalogue, input: &str) -> Result<Outcome, String> {
        self.execute_inner(c, input, true, None)
    }
    /// Opt-in guest-image boundary: callers derive object presence from their
    /// source-bound virtual files, never the host filesystem. Loaded modules may
    /// outlive their object files. Existing consoles retain their full supplied
    /// object universe and byte-for-byte state shape through the methods above.
    pub fn execute_available(
        &mut self,
        c: &Catalogue,
        input: &str,
        present: &BTreeSet<String>,
    ) -> Result<Outcome, String> {
        if present.len() > c.modules.len() || present.iter().any(|n| !c.modules.contains_key(n)) {
            return Err("Object presence may name only modules in the supplied catalogue.".into());
        }
        self.execute_inner(c, input, false, Some(present))
    }
    fn execute_inner(
        &mut self,
        c: &Catalogue,
        input: &str,
        named: bool,
        present: Option<&BTreeSet<String>>,
    ) -> Result<Outcome, String> {
        if input.len() > MAX_COMMAND {
            return Err("Module command exceeds 4,096 bytes.".into());
        }
        self.validate(c)?;
        let w = crate::shell::literal_command(input)?;
        let args: Vec<&str> = w.iter().map(String::as_str).collect();
        let out = match args.as_slice() {
            [] => Outcome::ok(""),
            ["help"] => Outcome::ok(if named { NAMED_HELP } else { HELP }),
            ["uname", "-r"] => Outcome::ok(format!("{}\n", c.kernel_release)),
            ["lspci", "-k"] => {
                let mut out = String::new();
                for d in &c.devices {
                    out.push_str(&format!("{} {}\n", d.slot, d.description));
                    if self.loaded.contains_key(&d.driver) {
                        out.push_str(&format!("\tKernel driver in use: {}\n", d.driver));
                    }
                    if present.is_none_or(|objects| objects.contains(&d.driver) && self.index.contains(&d.driver)) {
                        out.push_str(&format!("\tKernel modules: {}\n", d.driver));
                    }
                }
                Outcome::ok(out)
            }
            ["lsmod"] => {
                let mut out = String::from("Module                  Size  Used by\n");
                for n in self.loaded.keys() {
                    let users = self.users(c, n);
                    out.push_str(&format!(
                        "{n:<20} {:>7}  {} {}\n",
                        c.modules[n].size,
                        usize::from(c.modules[n].external_users) + users.len(),
                        users.join(",")
                    ));
                }
                Outcome::ok(out)
            }
            ["modinfo", which] if !which.starts_with('-') => {
                let n = if which.starts_with('/') {
                    c.object(which).map(str::to_owned)
                } else {
                    module_name(which).ok().filter(|n| self.index.contains(n))
                };
                let n = n.filter(|n| present.is_none_or(|objects| objects.contains(n)));
                match n {
                    None => Outcome::failed("modinfo: Module or supplied object not found."),
                    Some(n) => {
                        let m = &c.modules[&n];
                        let mut out = format!(
                            "filename:       {}\nname:           {n}\ndescription:    {}\ndepends:        {}\nvermagic:       {}\n",
                            m.path,
                            m.description,
                            m.dependencies.join(","),
                            m.kernel_release
                        );
                        for (k, p) in &m.parameters {
                            out.push_str(&format!(
                                "parm:           {k}:{} (int; {}..{}; default {})\n",
                                p.description, p.minimum, p.maximum, p.default
                            ));
                        }
                        Outcome::ok(out)
                    }
                }
            }
            ["depmod", "-a"] => {
                self.index = c
                    .modules
                    .iter()
                    .filter(|(n, m)| m.kernel_release == c.kernel_release
                        && present.is_none_or(|objects| objects.contains(*n)))
                    .map(|(n, _)| n.clone())
                    .collect();
                Outcome::ok("")
            }
            ["dmesg"] => Outcome::ok(if self.diagnostics.is_empty() {
                String::new()
            } else {
                format!("{}\n", self.diagnostics.join("\n"))
            }),
            ["cat", p] if *p == CONFIG_PATH => Outcome::ok(self.config.clone()),
            ["cat", p] => {
                let parts: Vec<_> = p.split('/').collect();
                if let ["", "sys", "module", n, "parameters", key] = parts.as_slice() {
                    let n = module_name(n)?;
                    match self.loaded.get(&n).and_then(|m| m.parameters.get(*key)) {
                        Some(v) => Outcome::ok(format!("{v}\n")),
                        None => Outcome::failed(
                            "cat: supplied effective parameter does not exist; the module may not be loaded.",
                        ),
                    }
                } else {
                    return Err("Only the supplied options file and /sys/module/NAME/parameters/PARAMETER are supported virtual reads.".into());
                }
            }
            ["modprobe", "-r", n] | ["rmmod", n] if !n.starts_with('-') => {
                let n = module_name(n)?;
                if !c.modules.contains_key(&n) {
                    if named {
                        return Err("This named-load subset removes physical module names only; inspect lsmod, then remove its physical name. Alias removal is unsupported.".into());
                    }
                    Outcome::failed(format!("Module {n} is not supplied."))
                } else {
                    match self.remove(c, &n, args[0] == "modprobe") {
                        Ok(()) => Outcome::ok(""),
                        Err(e) => Outcome::failed(e),
                    }
                }
            }
            [cmd, n, rest @ ..] if matches!(*cmd, "modprobe" | "insmod") && !n.starts_with('-') => {
                let extra = assignments(&rest.iter().map(|s| s.to_string()).collect::<Vec<_>>())?;
                let named_config = if named && *cmd == "modprobe" {
                    Some(parse_named_config(&self.config, c)?)
                } else {
                    None
                };
                let n = if *cmd == "insmod" {
                    c.object(n)
                        .map(str::to_owned)
                        .ok_or("insmod requires one exact supplied virtual object path.")?
                } else {
                    module_name(n)?
                };
                let n = named_config
                    .as_ref()
                    .and_then(|cfg| cfg.aliases.get(&n))
                    .cloned()
                    .unwrap_or(n);
                if !c.modules.contains_key(&n) {
                    Outcome::failed(if named {
                        format!(
                            "Module {n} is not supplied or configured as an exact supported name."
                        )
                    } else {
                        format!("Module {n} is not supplied.")
                    })
                } else {
                    known_options(c, &n, &extra)?;
                    let result = if *cmd == "insmod" {
                        self.insert(c, &n, &extra, present)
                    } else {
                        let saved = if let Some(config) = named_config {
                            config.options
                        } else {
                            parse_config(&self.config, c)?
                        };
                        for (module, options) in &saved {
                            known_options(c, module, options)?;
                        }
                        if saved
                            .get(&n)
                            .is_some_and(|options| extra.keys().any(|k| options.contains_key(k)))
                        {
                            return Err("Duplicate saved/command-line scalar options are outside this subset; keep a single source for each value.".into());
                        }
                        self.load(c, &n, &saved, &extra, present)
                    };
                    match result {
                        Ok(()) => Outcome::ok(""),
                        Err(e) => Outcome::failed(format!("{cmd}: {e}")),
                    }
                }
            }
            ["alias" | "options", ..] if named => return Err("This is a modprobe.d configuration line, not a command in this console. Edit and save the supplied file in the native editor, then use modprobe to request a name.".into()),
            _ => {
                return Err(format!(
                    "Unsupported module-console syntax. {}",
                    if named { NAMED_HELP } else { HELP }
                ));
            }
        };
        self.validate(c)?;
        Ok(out)
    }
}
