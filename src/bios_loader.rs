//! Finite BIOS/MBR loader installation over existing virtual geometry.
//! No sector bytes, filesystem calls, boot programs, subprocesses or firmware APIs.
use crate::{boot_entry as boot, grub_menu, partition_fs as disk};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, boot::Error>;
pub const HELP: &str = "BIOS/MBR supplied-machine subset: grub-install [--target=i386-pc] [--boot-directory=PATH] WHOLE_DISK, with either --option=value or separate option/value words; lsblk -o NAME,TYPE,FSTYPE,MOUNTPOINTS; findmnt -T SUPPLIED_BOOT_DIRECTORY; cat SUPPLIED_BOOT_DIRECTORY/grub/grub.cfg. Defaults are the supplied i386-pc package and /boot. Literal quotes and escapes are supported; duplicate/unknown options, shell expansion, pipes and compound commands are refused. Test boot chain is a native simulator action, not a Linux command. Supplied mounted ext4 boot directories and an exact embedding-space requirement are preconditions. Installs change only typed boot-code/module-location records and preserve every partition/filesystem value. No blocklists, force, BIOS/GPT, EFI, Legacy, package installation, raw bytes, actual GRUB, mount operations or host access. Simulator refusal/atomicity is not real-tool protection.";
fn error(kind: boot::ErrorKind, message: impl Into<String>) -> boot::Error {
    boot::Error {
        kind,
        message: message.into(),
    }
}
fn plain(s: &str, n: usize) -> bool {
    !s.is_empty() && s.len() <= n && !s.chars().any(char::is_control)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub menu: String,
    pub inventory: boot::Inventory,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Directory {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub disk: String,
    pub partition: u8,
    pub writable: bool,
    /// None means an existing mounted directory with no supplied GRUB menu.
    pub configuration: Option<Configuration>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub geometry: disk::State,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub boot_disk: String,
    /// Explicit total reservation for this supplied core image, not a universal size.
    pub core_sectors: u32,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Directory,8>")]
    pub directories: Vec<Directory>,
}
impl Facts {
    pub fn validate(&self) -> Result<()> {
        self.geometry
            .validate()
            .map_err(|e| error(boot::ErrorKind::Invalid, e.to_string()))?;
        if !self.geometry.disks.contains_key(&self.boot_disk)
            || !(1..=8192).contains(&self.core_sectors)
            || self.directories.is_empty()
            || self.directories.len() > 8
        {
            return Err(error(
                boot::ErrorKind::Invalid,
                "Supply a boot disk, 1–8192 core-image sectors and 1–8 mounted boot directories.",
            ));
        }
        let mut paths = BTreeSet::new();
        for directory in &self.directories {
            let fs = self
                .geometry
                .disks
                .get(&directory.disk)
                .and_then(|d| d.partitions().find(|p| p.number == directory.partition));
            if !boot::path(&directory.path)
                || !paths.insert(&directory.path)
                || !fs.is_some_and(|p| matches!(p.content, disk::Content::Ext4 { .. }))
            {
                return Err(error(
                    boot::ErrorKind::Invalid,
                    "Each mounted boot directory needs a unique normalized path and an existing supplied ext4 partition.",
                ));
            }
            if let Some(c) = &directory.configuration {
                c.inventory.validate()?;
                grub_menu::Menu::parse(&c.menu)?;
                if c.inventory.roots.iter().any(|root|!self.geometry.disks.values().flat_map(|d|d.partitions()).any(|p|matches!(&p.content,disk::Content::Ext4{uuid,..}if uuid.eq_ignore_ascii_case(&root.uuid)))) {
                    return Err(error(boot::ErrorKind::Invalid,"Supplied system-root image facts must refer to an existing virtual ext4 filesystem UUID."));
                }
            }
        }
        Ok(())
    }
    fn directory(&self, path: &str) -> Result<&Directory> {
        self.directories.iter().find(|d|d.path==path).ok_or_else(||error(boot::ErrorKind::Invalid,"That boot directory is not among the supplied mounted locations. Use findmnt -T with a listed path."))
    }
    pub fn embedding_sectors(&self, path: &str) -> Result<u64> {
        let d = self.geometry.disks.get(path).ok_or_else(|| {
            error(
                boot::ErrorKind::Invalid,
                "Install to a supplied whole-disk path, not a filesystem partition.",
            )
        })?;
        let table=d.mbr.as_ref().ok_or_else(||error(boot::ErrorKind::Unsupported,"This profile models BIOS/MBR embedding only; GPT/EFI installation needs a different model."))?;
        let first=table.entries.iter().map(|e|e.partition.start).min().ok_or_else(||error(boot::ErrorKind::Unsupported,"This profile needs a supplied partitioned MBR disk; whole unpartitioned targets are outside its scope."))?;
        Ok(first.saturating_sub(1))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Code {
    Other {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        label: String,
    },
    GrubBios {
        #[serde(deserialize_with = "crate::cron::text::<_,128>")]
        boot_directory: String,
        core_sectors: u32,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub disk: String,
    pub code: Code,
}
fn observation<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<Vec<Record>>, D::Error> {
    #[derive(Deserialize)]
    struct Rows(
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Record,4>")] Vec<Record>,
    );
    Option::<Rows>::deserialize(d).map(|v| v.map(|r| r.0))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Record,4>")]
    pub records: Vec<Record>,
    #[serde(default, deserialize_with = "observation")]
    pub checked: Option<Vec<Record>>,
}
impl Default for World {
    fn default() -> Self {
        Self {
            version: 1,
            records: vec![],
            checked: None,
        }
    }
}
impl World {
    pub fn shape(&self) -> Result<()> {
        if self.version != 1
            || self.records.len() > 4
            || self.checked.as_ref().is_some_and(|v| v != &self.records)
        {
            return Err(error(
                boot::ErrorKind::Invalid,
                "Saved boot records or their observation are incompatible; original retained.",
            ));
        }
        let mut seen = BTreeSet::new();
        for record in &self.records {
            if !plain(&record.disk, 64) || !seen.insert(&record.disk) {
                return Err(error(
                    boot::ErrorKind::Invalid,
                    "Boot records need bounded unique device identities.",
                ));
            }
            match &record.code {
                Code::Other { label } => {
                    if !plain(label, 64) {
                        return Err(error(
                            boot::ErrorKind::Invalid,
                            "Invalid other-loader label.",
                        ));
                    }
                }
                Code::GrubBios {
                    boot_directory,
                    core_sectors,
                } => {
                    if !boot::path(boot_directory) || !(1..=8192).contains(core_sectors) {
                        return Err(error(
                            boot::ErrorKind::Invalid,
                            "Invalid saved directory/core reservation.",
                        ));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn execute(&mut self, facts: &Facts, input: &str) -> Result<String> {
        self.bound(facts)?;
        let args = words(input)?;
        let args = args.iter().map(String::as_str).collect::<Vec<_>>();
        match args.as_slice() {
            ["grub-install", ..] => self.install(facts, &Install::parse(input)?),
            ["lsblk", "-o", "NAME,TYPE,FSTYPE,MOUNTPOINTS"] => {
                let mut out = String::from("NAME TYPE FSTYPE MOUNTPOINTS\n");
                for (name, d) in &facts.geometry.disks {
                    out.push_str(&format!("{} disk\n", name.trim_start_matches("/dev/")));
                    for p in d.partitions() {
                        let kind = match p.content {
                            disk::Content::Ext4 { .. } => "ext4",
                            disk::Content::Swap { .. } => "swap",
                            _ => "",
                        };
                        let mounted = facts
                            .directories
                            .iter()
                            .filter(|dir| dir.disk == *name && dir.partition == p.number)
                            .map(|dir| dir.path.as_str())
                            .collect::<Vec<_>>()
                            .join(",");
                        out.push_str(&format!(
                            "  {}{} part {kind} {mounted}\n",
                            name.trim_start_matches("/dev/"),
                            p.number
                        ));
                    }
                }
                Ok(out)
            }
            ["findmnt", "-T", path] => {
                let dir = facts.directory(path)?;
                Ok(format!(
                    "TARGET SOURCE FSTYPE OPTIONS\n{} {}{} ext4 {}\n",
                    dir.path,
                    dir.disk,
                    dir.partition,
                    if dir.writable && facts.geometry.disks[&dir.disk].writable {
                        "rw"
                    } else {
                        "ro"
                    }
                ))
            }
            ["cat", path] => {
                let dir=facts.directories.iter().find(|d|*path==format!("{}/grub/grub.cfg",d.path)).ok_or_else(||error(boot::ErrorKind::Unsupported,"cat reads only a supplied boot directory's grub/grub.cfg; no host path is opened."))?;
                dir.configuration.as_ref().map(|c|c.menu.clone()).ok_or_else(||error(boot::ErrorKind::Invalid,"The supplied directory has no grub/grub.cfg. Installation alone does not generate it."))
            }
            _ => Err(error(
                boot::ErrorKind::Unsupported,
                "Command or options are outside this supplied BIOS/MBR subset. Open Supported commands; nothing was executed on the host.",
            )),
        }
    }
    pub fn bound(&self, facts: &Facts) -> Result<()> {
        facts.validate()?;
        self.shape()?;
        if self.version != 1
            || self.records.len() > 4
            || self.checked.as_ref().is_some_and(|v| v != &self.records)
        {
            return Err(error(
                boot::ErrorKind::Invalid,
                "Saved boot records or their observation are incompatible; original retained.",
            ));
        }
        let mut seen = BTreeSet::new();
        for r in &self.records {
            if !seen.insert(&r.disk) || !facts.geometry.disks.contains_key(&r.disk) {
                return Err(error(
                    boot::ErrorKind::Invalid,
                    "Boot records must name unique supplied disks.",
                ));
            }
            match &r.code {
                Code::Other { label } => {
                    if !plain(label, 64) {
                        return Err(error(
                            boot::ErrorKind::Invalid,
                            "A supplied other-loader label is invalid.",
                        ));
                    }
                }
                Code::GrubBios {
                    boot_directory,
                    core_sectors,
                } => {
                    facts.directory(boot_directory)?;
                    if *core_sectors != facts.core_sectors
                        || facts.embedding_sectors(&r.disk)? < u64::from(*core_sectors)
                    {
                        return Err(error(
                            boot::ErrorKind::Invalid,
                            "Saved core placement does not fit the supplied image and disk geometry.",
                        ));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn install(&mut self, facts: &Facts, request: &Install) -> Result<String> {
        self.bound(facts)?;
        if request.target != "i386-pc" {
            return Err(error(
                boot::ErrorKind::Unsupported,
                "Only the supplied i386-pc BIOS package is modeled; this is not an EFI installation.",
            ));
        }
        let target=facts.geometry.disks.get(&request.disk).ok_or_else(||error(boot::ErrorKind::Invalid,"Use a supplied whole disk, such as /dev/vda; partition installation/blocklists are outside this profile."))?;
        let directory = facts.directory(&request.boot_directory)?;
        let directory_disk = &facts.geometry.disks[&directory.disk];
        if !target.writable || !directory.writable || !directory_disk.writable {
            return Err(error(
                boot::ErrorKind::Invalid,
                "The target disk and supplied mounted boot directory must be writable. Nothing changed.",
            ));
        }
        let available = facts.embedding_sectors(&request.disk)?;
        if available < u64::from(facts.core_sectors) {
            return Err(error(
                boot::ErrorKind::Invalid,
                format!(
                    "Only {available} sectors are free after the MBR; this supplied core needs {}. Blocklist/force fallback is not modeled. Nothing changed.",
                    facts.core_sectors
                ),
            ));
        }
        let record = Record {
            disk: request.disk.clone(),
            code: Code::GrubBios {
                boot_directory: directory.path.clone(),
                core_sectors: facts.core_sectors,
            },
        };
        if let Some(slot) = self.records.iter_mut().find(|r| r.disk == request.disk) {
            *slot = record;
        } else {
            self.records.push(record);
        }
        self.checked = None;
        Ok("Installing for i386-pc platform.\nInstallation finished. No error reported.\n".into())
    }
    pub fn test(&mut self, facts: &Facts) -> Result<Trace> {
        self.bound(facts)?;
        let result = self.trace(facts)?;
        self.checked = Some(self.records.clone());
        Ok(result)
    }
    pub fn trace(&self, facts: &Facts) -> Result<Trace> {
        self.bound(facts)?;
        let mut trace = Trace {
            stages: vec![boot::Stage {
                title: "Firmware selection",
                passed: true,
                detail: format!(
                    "The supplied BIOS starts from {}. No firmware was run.",
                    facts.boot_disk
                ),
            }],
            handoff: None,
        };
        let Some(record) = self.records.iter().find(|r| r.disk == facts.boot_disk) else {
            trace.stage("MBR boot code",false,"The selected disk has no installed boot code in this model. A successful installation on another disk cannot repair this path.");
            return Ok(trace);
        };
        let Code::GrubBios {
            boot_directory,
            core_sectors,
        } = &record.code
        else {
            trace.stage(
                "MBR boot code",
                false,
                "The selected disk has another loader, not the requested GRUB BIOS path.",
            );
            return Ok(trace);
        };
        trace.stage("MBR boot code",true,"The selected disk points to the installed GRUB core; its partition table is preserved.");
        trace.stage("Embedded core",true,format!("The supplied {core_sectors}-sector core fits the post-MBR area and uses {boot_directory}/grub for modules/configuration."));
        let directory = facts.directory(boot_directory)?;
        let Some(configuration) = &directory.configuration else {
            trace.stage("Menu configuration",false,"The chosen boot directory has no supplied grub/grub.cfg. grub-install installs loader files; it does not generate a menu. Select the supplied configured boot directory.");
            return Ok(trace);
        };
        trace.stage(
            "Menu configuration",
            true,
            format!("Read the supplied {boot_directory}/grub/grub.cfg."),
        );
        let report = grub_menu::test(&configuration.inventory, &configuration.menu)?;
        trace.stages.extend(report.stages.iter().cloned());
        trace.handoff = Some(report);
        Ok(trace)
    }
    pub fn current_test(&self) -> bool {
        self.checked.as_ref() == Some(&self.records)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    pub stages: Vec<boot::Stage>,
    pub handoff: Option<boot::Report>,
}
impl Trace {
    fn stage(&mut self, title: &'static str, passed: bool, detail: impl Into<String>) {
        self.stages.push(boot::Stage {
            title,
            passed,
            detail: detail.into(),
        });
    }
    pub fn passed(&self) -> bool {
        self.stages.iter().all(|s| s.passed)
            && self.handoff.as_ref().is_some_and(boot::Report::passed)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Install {
    pub target: String,
    pub boot_directory: String,
    pub disk: String,
}
impl Install {
    pub fn parse(raw: &str) -> Result<Self> {
        let words = words(raw)?;
        if words.first().map(String::as_str) != Some("grub-install") {
            return Err(error(
                boot::ErrorKind::Unsupported,
                "Supported install command is grub-install; no host command was executed.",
            ));
        }
        let mut target = None;
        let mut directory = None;
        let mut disk = None;
        let mut at = 1;
        while at < words.len() {
            let word = &words[at];
            if word.starts_with('-') {
                let (key, inline) = if let Some((k, v)) = word.split_once('=') {
                    (k, Some(v))
                } else {
                    (word.as_str(), None)
                };
                if !matches!(key, "--target" | "--boot-directory") {
                    return Err(error(
                        boot::ErrorKind::Unsupported,
                        format!(
                            "Option {key:?} is outside the BIOS/MBR subset; nothing was installed."
                        ),
                    ));
                }
                let value = if let Some(value) = inline {
                    value
                } else {
                    at += 1;
                    words
                        .get(at)
                        .ok_or_else(|| error(boot::ErrorKind::Syntax, "The option needs a value."))?
                        .as_str()
                };
                let slot = match key {
                    "--target" => &mut target,
                    "--boot-directory" => &mut directory,
                    _ => {
                        return Err(error(
                            boot::ErrorKind::Unsupported,
                            format!(
                                "Option {key:?} is outside the BIOS/MBR subset; nothing was installed."
                            ),
                        ));
                    }
                };
                if value.is_empty() || slot.replace(value.to_owned()).is_some() {
                    return Err(error(
                        boot::ErrorKind::Unsupported,
                        "Empty or repeated install options are outside this subset.",
                    ));
                }
            } else if disk.replace(word.clone()).is_some() {
                return Err(error(
                    boot::ErrorKind::Syntax,
                    "Use one whole-disk installation target.",
                ));
            }
            at += 1;
        }
        let request = Self {
            target: target.unwrap_or_else(|| "i386-pc".into()),
            boot_directory: directory.unwrap_or_else(|| "/boot".into()),
            disk: disk.ok_or_else(|| {
                error(
                    boot::ErrorKind::Syntax,
                    "Supply the whole disk to receive BIOS boot code.",
                )
            })?,
        };
        if !boot::path(&request.boot_directory) || !plain(&request.disk, 64) {
            return Err(error(
                boot::ErrorKind::Invalid,
                "Use normalized supplied directory/device paths; traversal is not supported.",
            ));
        }
        Ok(request)
    }
}
/// Literal shell-word spelling only, not shell evaluation. Bound before allocation.
pub(crate) fn words(raw: &str) -> Result<Vec<String>> {
    if raw.len() > 1024 {
        return Err(error(
            boot::ErrorKind::Limit,
            "Keep one install command within 1024 bytes.",
        ));
    }
    if raw.bytes().any(|b| !(32..=126).contains(&b) && b != b'\t') {
        return Err(error(
            boot::ErrorKind::Unsupported,
            "Use one ordinary ASCII command line; multiline commands are outside this subset.",
        ));
    }
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut started = false;
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else if q == '"' && c == '\\' {
                let n = chars
                    .next()
                    .ok_or_else(|| error(boot::ErrorKind::Syntax, "Finish the quoted escape."))?;
                if matches!(n, '$' | '`' | '"' | '\\') {
                    word.push(n);
                } else {
                    word.push('\\');
                    word.push(n);
                }
            } else if q == '"' && matches!(c, '$' | '`') {
                return Err(error(
                    boot::ErrorKind::Unsupported,
                    "Shell expansion is outside the literal install subset.",
                ));
            } else {
                word.push(c);
            }
        } else {
            match c {
                '\'' | '"' => {
                    quote = Some(c);
                    started = true;
                }
                '\\' => {
                    word.push(chars.next().ok_or_else(|| {
                        error(boot::ErrorKind::Syntax, "Finish the command escape.")
                    })?);
                    started = true;
                }
                ' ' | '\t' => {
                    if started {
                        words.push(std::mem::take(&mut word));
                        started = false;
                    }
                }
                '$' | '`' | '~' | '#' | ';' | '&' | '|' | '<' | '>' | '(' | ')' | '*' | '?'
                | '[' | ']' | '{' | '}' => {
                    return Err(error(
                        boot::ErrorKind::Unsupported,
                        "Use literal supplied paths; expansions, globbing, operators and comments are outside this command subset.",
                    ));
                }
                _ => {
                    word.push(c);
                    started = true;
                }
            }
        }
        if words.len() > 24 {
            return Err(error(
                boot::ErrorKind::Limit,
                "Use at most 24 command words.",
            ));
        }
    }
    if quote.is_some() {
        return Err(error(
            boot::ErrorKind::Syntax,
            "Close the quoted command word.",
        ));
    }
    if started {
        words.push(word);
    }
    if words.len() > 24 {
        return Err(error(
            boot::ErrorKind::Limit,
            "Use at most 24 command words.",
        ));
    }
    Ok(words)
}
