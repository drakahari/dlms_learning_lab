//! Bounded supplied generic-initramfs operations; paths never reach host APIs.
use crate::{boot_entry, kernel_build as kernel, kernel_stage, source_install};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, String>;
pub const CONFIG_LIMIT: usize = 4096;
pub const INPUT_LIMIT: usize = 2048;
pub const HELP: &str = "Simulated generic early-userspace preparation: dracut --kver RELEASE OUTPUT or dracut OUTPUT RELEASE; -f/--force permits replacing an existing virtual initramfs. Use a supplied prepared kernel/module tree and an absolute output inside the displayed virtual boot directory. lsinitrd OUTPUT shows only modeled kernel-driver files/release, not real archive contents. dracut -k is a module-directory option, not --kver; lsinitrd -m lists dracut feature modules, not kernel drivers. These options, host/default discovery, extraction, compression, hooks and arbitrary shell/configuration evaluation are unsupported. No host tools or files are accessed.";
const DRIVERS: [&str; 5] = ["nvme_core", "nvme", "ext4", "jbd2", "mbcache"];
fn drivers<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeSet<String>, D::Error> {
    use serde::de::Error as _;
    #[derive(Deserialize)]
    struct Name(#[serde(deserialize_with = "crate::cron::text::<_,16>")] String);
    let names = crate::cron_workspace::sequence::<_, Name, 5>(d)?;
    let mut out = BTreeSet::new();
    for Name(name) in names {
        if !DRIVERS.contains(&name.as_str()) || !out.insert(name) {
            return Err(D::Error::custom(
                "Use each supplied kernel driver at most once.",
            ));
        }
    }
    Ok(out)
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(deserialize_with = "drivers")]
    pub omitted: BTreeSet<String>,
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.omitted.len() > 5 || self.omitted.iter().any(|n| !DRIVERS.contains(&n.as_str())) {
            return Err("This supplied generic profile supports only nvme_core, nvme, ext4, jbd2 and mbcache.".into());
        }
        Ok(())
    }
    pub fn parse(source: &str) -> Result<Self> {
        if source.len() > CONFIG_LIMIT
            || source.lines().count() > 32
            || !source.is_ascii()
            || source
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
        {
            return Err(
                "Keep the fragment within 4 KiB / 32 lines of plain text, with LF line endings."
                    .into(),
            );
        }
        let mut value = String::new();
        for (n, line) in source.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let error = |reason: &str| format!("Line {}: {reason}", n + 1);
            let rest=line.strip_prefix("omit_drivers").ok_or_else(||error("Only literal omit_drivers assignments are supported; other configuration/scripts are not evaluated."))?;
            let (append, rest) = if let Some(rest) = rest.strip_prefix("+=") {
                (true, rest)
            } else if let Some(rest) = rest.strip_prefix('=') {
                (false, rest)
            } else {
                return Err(error(
                    "Use omit_drivers=VALUE or omit_drivers+=VALUE without spaces around =.",
                ));
            };
            let (literal, tail) = if rest.starts_with(['\'', '"']) {
                let quote = rest.as_bytes()[0] as char;
                let end=rest[1..].find(quote).ok_or_else(||error("Close the literal quote; no expansion, escapes or command substitution are supported."))?+1;
                (&rest[1..end], &rest[end + 1..])
            } else {
                let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                (&rest[..end], &rest[end..])
            };
            if !(tail.is_empty()
                || tail.starts_with(char::is_whitespace) && tail.trim_start().starts_with('#'))
            {
                return Err(error(
                    "Use one literal value, optionally followed by a separated # comment. Shell statements are not evaluated.",
                ));
            }
            if !literal
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-. \t".contains(&b))
            {
                return Err(error(
                    "Use supplied literal driver names; expansion, shell punctuation and escapes are outside this subset.",
                ));
            }
            if !append {
                value.clear();
            }
            value.push_str(literal);
        }
        let mut omitted = BTreeSet::new();
        for name in value.split_ascii_whitespace() {
            if !DRIVERS.contains(&name) {
                return Err(format!(
                    "{name} is not a supplied driver name. += joins text exactly; include spaces between appended names."
                ));
            }
            omitted.insert(name.into());
        }
        let c = Self { omitted };
        c.validate()?;
        Ok(c)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Build {
        release: String,
        path: String,
        force: bool,
    },
    List {
        path: String,
    },
}
fn release(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && !s.starts_with('-')
        && !matches!(s, "." | "..")
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
}
pub fn parse(input: &str) -> Result<Command> {
    if input.len() > INPUT_LIMIT
        || !input.is_ascii()
        || input.chars().any(char::is_control)
        || input.bytes().any(|b| b"'\"\\;$|&><`(){}[]*?#".contains(&b))
    {
        return Err("Enter one literal supported command within 2,048 bytes. This console does not expand quotes, scripts, operators or control characters.".into());
    }
    let words: Vec<_> = input.split_ascii_whitespace().collect();
    if words.len() == 2
        && matches!(words[0], "dracut" | "lsinitrd")
        && matches!(words[1], "--help" | "-h")
    {
        return Ok(Command::Help);
    }
    if words.first() == Some(&"lsinitrd") {
        if words.get(1) == Some(&"-m") || words.get(1) == Some(&"--mod") {
            return Err("lsinitrd -m lists dracut feature modules, not kernel drivers. This initial inventory subset supports only lsinitrd followed by an explicit virtual image path.".into());
        }
        if words.len() != 2 || !boot_entry::path(words[1]) {
            return Err("Use lsinitrd with one absolute supplied virtual image path; defaults, filters and extraction are outside this subset.".into());
        }
        return Ok(Command::List {
            path: words[1].into(),
        });
    }
    if words.first() != Some(&"dracut") || words.len() > 8 {
        return Err(
            "Use a listed dracut or lsinitrd operation; no shell or other tool is run.".into(),
        );
    }
    let mut kernel = None;
    let mut positional = vec![];
    let mut force = false;
    let mut at = 1;
    while at < words.len() {
        let word = words[at];
        at += 1;
        match word {
        "-f"|"--force"=>{if force{return Err("Use the force flag once.".into());}force=true;},
        "--kver"=>{let value=words.get(at).ok_or("--kver needs a supplied kernel release.")?;at+=1;if kernel.is_some()||!release(value){return Err("Use one literal supplied kernel release after --kver.".into());}kernel=Some((*value).to_owned());},
        "-k"|"--kmoddir"=>return Err("In dracut, -k means a kernel-module directory, unlike dkms -k. This subset uses --kver RELEASE to select a kernel; module-directory overrides are not supported.".into()),
        value if value.starts_with('-')=>return Err("That option is outside this initial dracut subset. Supported: --kver, -f/--force and one explicit output (or output plus kernel release).".into()),
        value=>{if positional.len()==2{return Err("Supply one output and one kernel release, without extra arguments.".into());}positional.push(value);}
    }
    }
    let(path,kernel)=match(positional.as_slice(),kernel){([path],Some(kernel))=>(*path,kernel),([path,kernel],None)if release(kernel)=>(*path,(*kernel).to_owned()),_=>return Err("Real dracut can choose running-kernel/distribution defaults. This subset requires dracut --kver RELEASE OUTPUT, or dracut OUTPUT RELEASE, using supplied values.".into())};
    if !boot_entry::path(path) {
        return Err(
            "Use a canonical absolute virtual output path without .. or empty components.".into(),
        );
    }
    Ok(Command::Build {
        release: kernel,
        path: path.into(),
        force,
    })
}
fn within_boot(prefix: &str, path: &str) -> Result<()> {
    if !source_install::directory(prefix)
        || !boot_entry::path(path)
        || !path
            .strip_prefix(&format!("{}/", source_install::join(prefix, "boot")?))
            .is_some_and(|leaf| !leaf.contains('/'))
    {
        return Err("Use a canonical image path directly in the supplied virtual boot directory; nested directories and host paths are outside this subset.".into());
    }
    Ok(())
}
fn module_path(release: &str, name: &str) -> String {
    let leaf = match name {
        "nvme_core" => "drivers/nvme/host/nvme-core.ko",
        "nvme" => "drivers/nvme/host/nvme.ko",
        "ext4" => "fs/ext4/ext4.ko",
        "jbd2" => "fs/jbd2/jbd2.ko",
        "mbcache" => "fs/mbcache.ko",
        _ => unreachable!("Validated supplied driver"),
    };
    format!("usr/lib/modules/{release}/kernel/{leaf}")
}
pub fn execute(
    prefix: &str,
    config: &Config,
    store: &mut kernel_stage::Store,
    input: &str,
) -> Result<String> {
    config.validate()?;
    store.validate()?;
    let command = parse(input)?;
    match command {
        Command::Help => Ok(HELP.into()),
        Command::List { path } => {
            within_boot(prefix, &path)?;
            let file=store.initramfs.iter().find(|f|f.path==path).ok_or("That virtual initramfs does not exist. Generate it for the supplied kernel before inspection.")?;
            let mut out = format!(
                "Image: {path}\nKernel: {}\nModeled kernel-driver file inventory only (not full archive metadata):",
                file.image.image_build.release
            );
            for name in &file.image.included {
                out.push('\n');
                out.push_str(&module_path(&file.image.image_build.release, name));
            }
            if file.image.included.is_empty() {
                out.push_str("\n(no separately loadable driver files included)");
            }
            Ok(out)
        }
        Command::Build {
            release,
            path,
            force,
        } => {
            within_boot(prefix, &path)?;
            if !force && store.initramfs.iter().any(|f| f.path == path) {
                return Err("That virtual image already exists. Inspect it first; use -f or --force only when you intend to replace it.".into());
            }
            let boot_root = format!("{}/", source_install::join(prefix, "boot")?);
            let images = store
                .images
                .iter()
                .filter(|f| f.build.release == release && f.path.starts_with(&boot_root))
                .collect::<Vec<_>>();
            if images.len() != 1 {
                return Err("Select one uniquely supplied prepared kernel release. Host discovery and ambiguous source builds are outside this subset.".into());
            }
            let image = &images[0].build;
            let module_root = source_install::join(prefix, &format!("lib/modules/{release}"))?;
            let modules=store.modules.iter().find(|f|f.path==module_root).ok_or("This subset requires the supplied prepared module tree for that kernel. Missing build inputs or real tool failure side effects are not simulated.")?;
            let mut generated = kernel::Initramfs::from_builds(image, Some(&modules.build))?;
            generated
                .included
                .retain(|driver| !config.omitted.contains(driver));
            generated.validate_shape()?;
            let mut next = store.clone();
            let file = kernel_stage::InitramfsFile {
                path: path.clone(),
                image: generated,
            };
            if let Some(old) = next.initramfs.iter_mut().find(|f| f.path == path) {
                *old = file;
            } else {
                next.initramfs.push(file);
            }
            next.validate()?;
            *store = next;
            Ok(format!(
                "Generated simulated early userspace for {release} at {path}. Inspect its driver inventory and verify the selected boot entry; generation alone does not prove the system can start."
            ))
        }
    }
}
