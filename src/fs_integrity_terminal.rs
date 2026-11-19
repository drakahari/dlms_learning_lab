//! Literal virtual-device console. No filesystem, process, shell or network I/O.
use crate::fs_integrity::{self as core, Finding, Mode, Volume};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt};
pub const MAX_COMMAND: usize = 2048;
pub const MAX_OUTPUT: usize = 32768;
pub const MAX_SYSTEM: usize = 2_097_152;
pub const HELP: &str = "Virtual consistency console: findmnt [DEVICE|TARGET]; dumpe2fs -h DEVICE (limited stored summary); ls -li MOUNTPOINT[/lost+found]; cat MOUNTPOINT/FILE; umount MOUNTPOINT; mount DEVICE MOUNTPOINT; e2fsck or fsck.ext4 with -f and exactly one of -n (read-only), -p (preen), -y (explicit yes). Combined/separate flag forms are supported. Known supplied devices/targets only. This exercise refuses even read-only mounted checks because their real-world findings are unreliable. Fixed abstract metadata regions, regular files and root/lost+found only; no raw ext4, journals, sparse files, arbitrary directories, busy processes, kernel or host commands. Repair cannot recreate overwritten data. help is a simulator helper.";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Disk {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub device: String,
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub mountpoint: String,
    pub volume: Volume,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct System {
    pub version: u32,
    #[serde(deserialize_with = "crate::fs_integrity::sequence::<_,Disk,2>")]
    pub disks: Vec<Disk>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticKind {
    Syntax,
    Unsupported,
    Unknown,
    Policy,
    Invalid,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub status: u8,
    pub message: String,
}
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Diagnostic {}
type Result<T> = std::result::Result<T, Diagnostic>;
fn diagnostic(kind: DiagnosticKind, status: u8, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        kind,
        status,
        message: message.into(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Find(Option<String>),
    Summary(String),
    List(String),
    Cat(String),
    Unmount(String),
    Mount { device: String, target: String },
    Check { device: String, mode: Mode },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub stdout: String,
    pub status: u8,
    pub changed: bool,
}
fn output(mut stdout: String, status: u8, changed: bool) -> Output {
    if stdout.len() > MAX_OUTPUT {
        let mut end = MAX_OUTPUT - 64;
        while !stdout.is_char_boundary(end) {
            end -= 1
        }
        stdout.truncate(end);
        stdout.push_str("\n[Simulator output limit reached; remaining text omitted.]\n");
    }
    Output {
        stdout,
        status,
        changed,
    }
}
fn device(s: &str) -> bool {
    let Some(tail) = s.strip_prefix("/dev/vd") else {
        return false;
    };
    let b = tail.as_bytes();
    (2..=3).contains(&b.len())
        && b[0].is_ascii_lowercase()
        && b[1..].iter().all(u8::is_ascii_digit)
        && !tail[1..].starts_with('0')
        && tail[1..].parse::<u8>().is_ok_and(|n| (1..=16).contains(&n))
}
fn target(s: &str) -> bool {
    s.strip_prefix("/mnt/").is_some_and(|n| {
        !n.is_empty()
            && n.len() <= 48
            && !n.starts_with('.')
            && n.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
    })
}
impl System {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err(diagnostic(
                DiagnosticKind::Unsupported,
                1,
                "Unsupported virtual filesystem-console version",
            ));
        }
        if self.disks.is_empty()
            || self.disks.len() > 2
            || !crate::cron_workspace::within(self, MAX_SYSTEM)
        {
            return Err(diagnostic(
                DiagnosticKind::Limit,
                1,
                "One or two bounded virtual volumes are supported",
            ));
        }
        let mut devices = BTreeSet::new();
        let mut targets = BTreeSet::new();
        for disk in &self.disks {
            if !device(&disk.device)
                || !target(&disk.mountpoint)
                || !devices.insert(&disk.device)
                || !targets.insert(&disk.mountpoint)
            {
                return Err(diagnostic(
                    DiagnosticKind::Invalid,
                    1,
                    "Use distinct supplied virtual devices and literal /mnt/ leaf mountpoints; no host paths or traversal",
                ));
            }
            disk.volume.validate().map_err(core_error)?;
        }
        Ok(())
    }
    pub fn decode(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_SYSTEM {
            return Err(diagnostic(
                DiagnosticKind::Limit,
                1,
                "Virtual console data exceeds2MiB",
            ));
        }
        let s: Self = serde_json::from_slice(raw).map_err(|e| {
            diagnostic(
                DiagnosticKind::Invalid,
                1,
                format!("Invalid virtual filesystem data: {e}"),
            )
        })?;
        s.validate()?;
        Ok(s)
    }
    fn disk(&self, name: &str) -> Result<usize> {
        self.disks
            .iter()
            .position(|d| d.device == name)
            .ok_or_else(|| {
                diagnostic(
                    DiagnosticKind::Unknown,
                    1,
                    "That device is not supplied by this exercise. No host device is opened.",
                )
            })
    }
    fn mountpoint(&self, name: &str) -> Result<usize> {
        self.disks
            .iter()
            .position(|d| d.mountpoint == name)
            .ok_or_else(|| {
                diagnostic(
                    DiagnosticKind::Unknown,
                    1,
                    "That mountpoint is not supplied by this exercise. No host path is read.",
                )
            })
    }
    fn mounted_path<'a>(&'a self, path: &'a str) -> Result<(&'a Disk, &'a str)> {
        for d in &self.disks {
            if path == d.mountpoint
                || path
                    .strip_prefix(&d.mountpoint)
                    .is_some_and(|p| p.starts_with('/'))
            {
                if !d.volume.mounted {
                    return Err(diagnostic(
                        DiagnosticKind::Policy,
                        1,
                        "The virtual volume is unmounted; underlying mountpoint-directory contents are outside this exercise.",
                    ));
                }
                let relative = path
                    .strip_prefix(&d.mountpoint)
                    .unwrap()
                    .strip_prefix('/')
                    .unwrap_or("");
                if relative.split('/').any(|p| matches!(p, "." | "..")) {
                    return Err(diagnostic(
                        DiagnosticKind::Unsupported,
                        1,
                        "Path traversal and host fallback are not supported",
                    ));
                }
                return Ok((d, relative));
            }
        }
        Err(diagnostic(
            DiagnosticKind::Unknown,
            1,
            "Only paths within the supplied mounted volumes are modeled; no host path is read",
        ))
    }
    pub fn execute(&mut self, input: &str) -> Result<Output> {
        self.validate()?;
        let command = parse(input)?;
        match command {
            Command::Help => Ok(output(HELP.into(), 0, false)),
            Command::Find(filter) => {
                let rows = self
                    .disks
                    .iter()
                    .filter(|d| {
                        d.volume.mounted
                            && filter
                                .as_ref()
                                .is_none_or(|x| *x == d.device || *x == d.mountpoint)
                    })
                    .collect::<Vec<_>>();
                let mut text = String::new();
                if !rows.is_empty() {
                    text.push_str("TARGET SOURCE FSTYPE OPTIONS\n")
                }
                for d in &rows {
                    text.push_str(&format!("{} {} ext4 rw\n", d.mountpoint, d.device))
                }
                Ok(output(text, u8::from(rows.is_empty()), false))
            }
            Command::Summary(name) => {
                let d = &self.disks[self.disk(&name)?];
                let v = &d.volume;
                Ok(output(
                    format!(
                        "DLMS virtual ext-family summary (not raw ext4 metadata)\nDevice: {}\nState: {}\nBlock count: {}\nFree blocks (stored): {}\nInode count: {}\nFree inodes (stored): {}\n",
                        d.device,
                        if v.mounted { "Mounted" } else { "Unmounted" },
                        v.block_count,
                        v.free_blocks,
                        v.inode_count,
                        v.free_inodes
                    ),
                    0,
                    false,
                ))
            }
            Command::List(path) => {
                let (d, relative) = self.mounted_path(&path)?;
                if !matches!(relative, "" | "lost+found") {
                    return Err(diagnostic(
                        DiagnosticKind::Unsupported,
                        1,
                        "ls -li models only the supplied volume root or its lost+found directory",
                    ));
                }
                let mut lines = vec![];
                let mut total = 0;
                if relative.is_empty() {
                    lines.push("11 drwx------ 2 root root 4096 Jan  1 00:00 lost+found".into());
                    total += 8;
                }
                for (name, ino) in &d.volume.entries {
                    let visible = if relative.is_empty() {
                        (!name.contains('/')).then_some(name.as_str())
                    } else {
                        name.strip_prefix("lost+found/")
                    };
                    if let Some(name) = visible {
                        let Some(file) = d.volume.files.get(ino) else {
                            return Err(diagnostic(
                                DiagnosticKind::Unsupported,
                                1,
                                "A directory entry points to missing inode data; ls cannot invent its file metadata",
                            ));
                        };
                        total += file.blocks.len() * 8;
                        lines.push(format!(
                            "{ino} -rw-r--r-- {} root root {} Jan  1 00:00 {name}",
                            file.links,
                            file.content.len()
                        ));
                    }
                }
                Ok(output(
                    format!("total {total}\n{}\n", lines.join("\n")),
                    0,
                    false,
                ))
            }
            Command::Cat(path) => {
                let (d, relative) = self.mounted_path(&path)?;
                let ino = d.volume.entries.get(relative).ok_or_else(|| {
                    diagnostic(
                        DiagnosticKind::Unknown,
                        1,
                        "No modeled file has that directory name",
                    )
                })?;
                let file=d.volume.files.get(ino).ok_or_else(||diagnostic(DiagnosticKind::Unsupported,1,"This directory entry points to absent data; the simulator cannot recreate its contents"))?;
                Ok(output(file.content.clone(), 0, false))
            }
            Command::Unmount(path) => {
                let i = self.mountpoint(&path)?;
                let d = &mut self.disks[i];
                if !d.volume.mounted {
                    return Err(diagnostic(
                        DiagnosticKind::Policy,
                        1,
                        "That virtual volume is already unmounted",
                    ));
                }
                d.volume.mounted = false;
                Ok(output(
                    format!("Simulated unmount: {}\n", d.mountpoint),
                    0,
                    true,
                ))
            }
            Command::Mount { device, target } => {
                let i = self.disk(&device)?;
                if self.disks[i].mountpoint != target {
                    return Err(diagnostic(
                        DiagnosticKind::Policy,
                        1,
                        "Retain the supplied device/mountpoint association; other mounts are outside this exercise",
                    ));
                }
                let d = &mut self.disks[i];
                if d.volume.mounted {
                    return Err(diagnostic(
                        DiagnosticKind::Policy,
                        1,
                        "That virtual volume is already mounted",
                    ));
                }
                d.volume.mounted = true;
                Ok(output(
                    format!("Simulated mount: {} on {}.\n", d.device, d.mountpoint),
                    0,
                    true,
                ))
            }
            Command::Check { device, mode } => {
                let i = self.disk(&device)?;
                let d = &mut self.disks[i];
                let report = d.volume.check(mode).map_err(core_error)?;
                let mut text = format!("DLMS virtual consistency check: {}\n", d.device);
                for finding in &report.before {
                    text.push_str(&finding_text(finding));
                    text.push('\n');
                }
                if let Some(reason) = report.blocked {
                    text.push_str(&reason);
                    text.push('\n');
                }
                text.push_str(match report.status{0=>"Exit 0: no modeled errors.\n",1=>"Exit 1: modeled filesystem errors corrected. Run a read-only check to verify.\n",4=>"Exit 4: modeled errors remain uncorrected.\n",5=>"Exit 5: some errors corrected (1), others remain (4).\n",_=>unreachable!()});
                Ok(output(text, report.status, !report.corrections.is_empty()))
            }
        }
    }
}
fn core_error(e: core::Error) -> Diagnostic {
    diagnostic(
        match e.kind {
            core::ErrorKind::Invalid => DiagnosticKind::Invalid,
            core::ErrorKind::Unsupported => DiagnosticKind::Unsupported,
            core::ErrorKind::Limit => DiagnosticKind::Limit,
            core::ErrorKind::Mounted => DiagnosticKind::Policy,
        },
        8,
        e.message,
    )
}
pub(crate) fn finding_text(f: &Finding) -> String {
    match f {
        Finding::MissingInode { name, inode } => {
            format!("Directory name {name} points to missing inode {inode}.")
        }
        Finding::MultiplyClaimed { block, owners } => format!(
            "Data block {block} is claimed more than once: {owners:?}. The original ownership cannot be inferred."
        ),
        Finding::Unattached { inode, empty } => format!(
            "Inode {inode} has no directory name and {}.",
            if *empty {
                "no file data"
            } else {
                "retains file data"
            }
        ),
        Finding::LinkCount {
            inode,
            recorded,
            actual,
        } => {
            format!("Inode {inode}: recorded link count {recorded}; directory references {actual}.")
        }
        Finding::BlockBitmap { missing, extra } => format!(
            "Block allocation map: used blocks marked free {missing:?}; unreferenced blocks marked used {extra:?}."
        ),
        Finding::InodeBitmap { missing, extra } => format!(
            "Inode allocation map: present inodes marked free {missing:?}; absent inodes marked used {extra:?}."
        ),
        Finding::FreeBlocks { recorded, actual } => {
            format!("Stored free-block count {recorded}; graph requires {actual}.")
        }
        Finding::FreeInodes { recorded, actual } => {
            format!("Stored free-inode count {recorded}; graph requires {actual}.")
        }
    }
}
pub fn parse(input: &str) -> Result<Command> {
    if input.len() > MAX_COMMAND {
        return Err(diagnostic(DiagnosticKind::Limit, 16, "Command exceeds2KiB"));
    }
    if !input.is_ascii()
        || input.bytes().any(|b| b.is_ascii_control() && b != b'\t')
        || input.contains([
            ';', '|', '&', '>', '<', '`', '$', '\\', '\'', '"', '*', '?', '[', ']',
        ])
    {
        return Err(diagnostic(
            DiagnosticKind::Unsupported,
            16,
            "Use one literal supported command. Shell operators, expansion, patterns, quoting and scripts are not executed by this console.",
        ));
    }
    let words = input.split_ascii_whitespace().collect::<Vec<_>>();
    match words.as_slice() {
        ["help"] => Ok(Command::Help),
        ["findmnt"] => Ok(Command::Find(None)),
        ["findmnt", path] => Ok(Command::Find(Some((*path).into()))),
        ["dumpe2fs", "-h", dev] => Ok(Command::Summary((*dev).into())),
        ["ls", "-li", path] => Ok(Command::List((*path).into())),
        ["cat", path] => Ok(Command::Cat((*path).into())),
        ["umount", path] => Ok(Command::Unmount((*path).into())),
        ["mount", dev, path] => Ok(Command::Mount {
            device: (*dev).into(),
            target: (*path).into(),
        }),
        [program, args @ ..] if matches!(*program, "e2fsck" | "fsck.ext4") => {
            let Some((dev, flags)) = args.split_last() else {
                return Err(diagnostic(
                    DiagnosticKind::Syntax,
                    16,
                    "Give explicit -f plus one check policy and a supplied virtual device",
                ));
            };
            if dev.starts_with('-') {
                return Err(diagnostic(
                    DiagnosticKind::Syntax,
                    16,
                    "Give a supplied virtual device after the check flags",
                ));
            }
            let mut force = false;
            let mut mode = None;
            for flag in flags {
                let Some(flag) = flag.strip_prefix('-').filter(|s| !s.is_empty()) else {
                    return Err(diagnostic(
                        DiagnosticKind::Unsupported,
                        16,
                        "Only -f with one of -n, -p or -y is supported",
                    ));
                };
                for c in flag.chars() {
                    match c {
                        'f' => force = true,
                        'n' | 'p' | 'y' => {
                            let next = match c {
                                'n' => Mode::ReadOnly,
                                'p' => Mode::Preen,
                                _ => Mode::Yes,
                            };
                            if mode.is_some_and(|m| m != next) {
                                return Err(diagnostic(
                                    DiagnosticKind::Syntax,
                                    16,
                                    "Read-only, preen and yes are mutually exclusive policies",
                                ));
                            }
                            mode = Some(next);
                        }
                        _ => {
                            return Err(diagnostic(
                                DiagnosticKind::Unsupported,
                                16,
                                "That e2fsck option is outside this model; no scan or repair was performed",
                            ));
                        }
                    }
                }
            }
            if !force || mode.is_none() {
                return Err(diagnostic(
                    DiagnosticKind::Unsupported,
                    16,
                    "This exercise requires -f and one of -n, -p or -y. Implicit clean-state skipping and journal behavior are not modeled",
                ));
            }
            Ok(Command::Check {
                device: (*dev).into(),
                mode: mode.unwrap(),
            })
        }
        [] => Err(diagnostic(
            DiagnosticKind::Syntax,
            16,
            "Enter a supported command; help lists this console's subset",
        )),
        _ => Err(diagnostic(
            DiagnosticKind::Unsupported,
            16,
            "Unsupported command or argument form. Use help for this virtual console; no host command is executed",
        )),
    }
}
