//! Bounded Type #1 boot-entry and early root handoff model. No host/firmware APIs.
//! Paths are literal keys relative to a supplied virtual $BOOT, never OS paths.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_SOURCE: usize = 4096;
pub const MAX_LINES: usize = 48;
pub const HELP: &str = "Type #1 boot-entry subset: title, version, linux, initrd and options. Values are literal; no shell/GRUB expansion. Repeat initrd and options lines concatenate in order. Duplicate title/version/linux is refused rather than assigned ambiguous semantics. Supported kernel options: one root=UUID=...; ro or rw; quiet; systemd.unit=multi-user.target or rescue.target. No device/PARTUUID roots, duplicate scalar kernel options, quoted kernel arguments, firmware/GRUB installation, menu ordering, Secure Boot, image loading, actual init execution or host files. Kernel/image/driver facts are supplied in the lab. Test entry computes a finite hypothetical handoff; it never reboots or changes the current system.";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Syntax,
    Unsupported,
    Limit,
    Invalid,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
fn fail(kind: ErrorKind, text: impl Into<String>) -> Error {
    Error {
        kind,
        message: text.into(),
    }
}
type Result<T> = std::result::Result<T, Error>;
fn plain(s: &str, n: usize) -> bool {
    !s.is_empty() && s.len() <= n && !s.chars().any(char::is_control)
}
fn name(s: &str) -> bool {
    plain(s, 64)
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
pub fn path(s: &str) -> bool {
    s.starts_with('/')
        && s.len() <= 128
        && s.len() > 1
        && s[1..].split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
        })
}
fn uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
fn texts<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,64>")] String);
    crate::cron_workspace::sequence::<_, Text, N>(d).map(|v| v.into_iter().map(|t| t.0).collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kernel {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub release: String,
    #[serde(deserialize_with = "texts::<_,8>")]
    pub builtin_drivers: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Initramfs {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub release: String,
    #[serde(deserialize_with = "texts::<_,8>")]
    pub drivers: Vec<String>,
    /// A supplied image's early userspace can resolve filesystem UUIDs.
    pub uuid_lookup: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Root {
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub uuid: String,
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub label: String,
    #[serde(deserialize_with = "texts::<_,8>")]
    pub required_drivers: Vec<String>,
    pub init_present: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Kernel,8>")]
    pub kernels: Vec<Kernel>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Initramfs,8>")]
    pub initramfs: Vec<Initramfs>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Root,4>")]
    pub roots: Vec<Root>,
}
fn drivers(v: &[String]) -> bool {
    v.len() <= 8 && v.iter().all(|s| name(s)) && v.iter().collect::<BTreeSet<_>>().len() == v.len()
}
impl Inventory {
    pub fn validate(&self) -> Result<()> {
        if self.kernels.is_empty()
            || self.kernels.len() > 8
            || self.initramfs.len() > 8
            || self.roots.is_empty()
            || self.roots.len() > 4
        {
            return Err(fail(
                ErrorKind::Limit,
                "Boot inventory exceeds its finite image/root limits.",
            ));
        }
        let mut paths = BTreeSet::new();
        for k in &self.kernels {
            if !path(&k.path)
                || !paths.insert(&k.path)
                || !name(&k.release)
                || !drivers(&k.builtin_drivers)
            {
                return Err(fail(
                    ErrorKind::Invalid,
                    "Invalid kernel path, release or driver facts.",
                ));
            }
        }
        for i in &self.initramfs {
            if !path(&i.path) || !paths.insert(&i.path) || !name(&i.release) || !drivers(&i.drivers)
            {
                return Err(fail(
                    ErrorKind::Invalid,
                    "Invalid initramfs path, release or driver facts.",
                ));
            }
        }
        let mut uuids = BTreeSet::new();
        for r in &self.roots {
            if !uuid(&r.uuid)
                || !uuids.insert(r.uuid.to_ascii_lowercase())
                || !plain(&r.label, 80)
                || r.required_drivers.is_empty()
                || !drivers(&r.required_drivers)
            {
                return Err(fail(
                    ErrorKind::Invalid,
                    "Invalid or duplicate filesystem UUID, label or required drivers.",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub title: Option<String>,
    pub version: Option<String>,
    pub linux: String,
    pub initrd: Vec<String>,
    pub options: Vec<String>,
}
impl Entry {
    pub fn parse(source: &str) -> Result<Self> {
        if source.len() > MAX_SOURCE || source.lines().count() > MAX_LINES {
            return Err(fail(
                ErrorKind::Limit,
                "Keep the boot entry within 48 lines and 4 KiB.",
            ));
        }
        if source
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
        {
            return Err(fail(
                ErrorKind::Syntax,
                "Use UTF-8 text with LF newlines; control characters are not supported.",
            ));
        }
        let mut e = Self {
            title: None,
            version: None,
            linux: String::new(),
            initrd: Vec::new(),
            options: Vec::new(),
        };
        for (n, line) in source.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line.split_once([' ', '\t']).ok_or_else(|| {
                fail(
                    ErrorKind::Syntax,
                    format!(
                        "Line {}: separate the key and its value with a space, not =.",
                        n + 1
                    ),
                )
            })?;
            let value = value.trim();
            if value.is_empty() {
                return Err(fail(
                    ErrorKind::Syntax,
                    format!("Line {}: {key} needs a value.", n + 1),
                ));
            }
            match key {
                "title" | "version" => {
                    let slot = if key == "title" {
                        &mut e.title
                    } else {
                        &mut e.version
                    };
                    if slot.is_some() {
                        return Err(fail(
                            ErrorKind::Unsupported,
                            format!("Line {}: repeated {key} is outside this subset.", n + 1),
                        ));
                    }
                    if !plain(value, 120) {
                        return Err(fail(
                            ErrorKind::Limit,
                            "Keep each title/version within120 readable bytes.",
                        ));
                    }
                    *slot = Some(value.into());
                }
                "linux" => {
                    if !e.linux.is_empty() {
                        return Err(fail(
                            ErrorKind::Unsupported,
                            "Repeated linux keys are outside this subset.",
                        ));
                    }
                    if !path(value) {
                        return Err(fail(
                            ErrorKind::Unsupported,
                            "linux needs one normalized literal path relative to virtual $BOOT; no traversal, URL, quotes or expansion.",
                        ));
                    }
                    e.linux = value.into();
                }
                "initrd" => {
                    if e.initrd.len() >= 4 {
                        return Err(fail(
                            ErrorKind::Limit,
                            "At most four ordered initramfs images.",
                        ));
                    }
                    if !path(value) {
                        return Err(fail(
                            ErrorKind::Unsupported,
                            "Each initrd line needs one normalized literal virtual $BOOT path.",
                        ));
                    }
                    e.initrd.push(value.into());
                }
                "options" => {
                    for token in value.split_whitespace() {
                        if e.options.len() >= 16 || token.len() > 128 {
                            return Err(fail(
                                ErrorKind::Limit,
                                "At most16 short kernel arguments.",
                            ));
                        }
                        e.options.push(token.into());
                    }
                }
                _ => {
                    return Err(fail(
                        ErrorKind::Unsupported,
                        format!(
                            "Line {}: {key} is not supported in this Type #1 boot-entry activity. No shell or GRUB commands run here.",
                            n + 1
                        ),
                    ));
                }
            }
        }
        if e.linux.is_empty() {
            return Err(fail(
                ErrorKind::Syntax,
                "Add a linux line naming a supplied kernel image.",
            ));
        }
        Ok(e)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub root: String,
    pub read_only: bool,
    pub target: String,
}
impl Options {
    pub fn parse(tokens: &[String]) -> Result<Self> {
        let mut root = None;
        let mut read_only = true;
        let mut target = None;
        for t in tokens {
            if let Some(v) = t.strip_prefix("root=UUID=") {
                if !uuid(v) {
                    return Err(fail(
                        ErrorKind::Syntax,
                        "root=UUID= needs a complete filesystem UUID.",
                    ));
                }
                if root.replace(v.to_ascii_lowercase()).is_some() {
                    return Err(fail(
                        ErrorKind::Unsupported,
                        "Repeated root= options are deliberately unsupported; keep one root choice.",
                    ));
                }
            } else if t == "ro" {
                read_only = true;
            } else if t == "rw" {
                read_only = false;
            } else if t == "quiet" {
            } else if let Some(v) = t.strip_prefix("systemd.unit=") {
                if !matches!(v, "multi-user.target" | "rescue.target") {
                    return Err(fail(
                        ErrorKind::Unsupported,
                        "Only multi-user.target and rescue.target are modeled here.",
                    ));
                }
                if target.replace(v.into()).is_some() {
                    return Err(fail(
                        ErrorKind::Unsupported,
                        "Repeated systemd.unit= arguments are outside this subset.",
                    ));
                }
            } else {
                return Err(fail(
                    ErrorKind::Unsupported,
                    format!(
                        "Unsupported kernel argument {t:?}; see the disclosed subset. It was not executed or silently ignored."
                    ),
                ));
            }
        }
        Ok(Self {
            root: root.ok_or_else(|| {
                fail(
                    ErrorKind::Syntax,
                    "Add one root=UUID= argument to identify the intended filesystem.",
                )
            })?,
            read_only,
            target: target.unwrap_or_else(|| "multi-user.target".into()),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stage {
    pub title: &'static str,
    pub passed: bool,
    pub detail: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub requested: Options,
    pub stages: Vec<Stage>,
    pub root: Option<String>,
    pub release: Option<String>,
    pub read_only: bool,
    pub target: Option<String>,
}
impl Report {
    pub fn passed(&self) -> bool {
        self.stages
            .last()
            .is_some_and(|s| s.passed && s.title == "Init handoff")
    }
    fn stage(&mut self, title: &'static str, passed: bool, detail: impl Into<String>) {
        self.stages.push(Stage {
            title,
            passed,
            detail: detail.into(),
        });
    }
}
pub fn test(inventory: &Inventory, source: &str) -> Result<Report> {
    inventory.validate()?;
    let e = Entry::parse(source)?;
    trace_entry(inventory, &e)
}
/// Shared pure handoff after a format-specific bounded parser has built an entry.
pub(crate) fn trace_entry(inventory: &Inventory, e: &Entry) -> Result<Report> {
    trace_entry_with_root_availability(inventory, e, |_| true)
}

pub(crate) fn trace_entry_with_root_availability(
    inventory: &Inventory,
    e: &Entry,
    root_available: impl FnOnce(&Root) -> bool,
) -> Result<Report> {
    inventory.validate()?;
    let o = Options::parse(&e.options)?;
    let mut r = Report {
        requested: o.clone(),
        stages: Vec::new(),
        root: None,
        release: None,
        read_only: o.read_only,
        target: None,
    };
    // Firmware, loader availability and the virtual boot partition are preconditions,
    // not fabricated evidence of implementing Secure Boot or a real firmware boot.
    let Some(k) = inventory.kernels.iter().find(|k| k.path == e.linux) else {
        r.stage("Kernel image",false,format!("The supplied boot partition has no kernel at {}. Compare the linux path with the image inventory.",e.linux));
        return Ok(r);
    };
    r.release = Some(k.release.clone());
    r.stage(
        "Kernel image",
        true,
        format!("Supplied kernel {} selected from {}.", k.release, k.path),
    );
    let mut available = k.builtin_drivers.iter().cloned().collect::<BTreeSet<_>>();
    let mut lookup = false;
    for path in &e.initrd {
        let Some(i) = inventory.initramfs.iter().find(|i| i.path == *path) else {
            r.stage("Early userspace",false,format!("No supplied initramfs exists at {path}. Check the initrd path, not the entry title."));
            return Ok(r);
        };
        if i.release != k.release {
            r.stage("Early userspace",false,format!("{path} supplies modules for {}, while the loaded kernel is {}. Select an image built for that kernel.",i.release,k.release));
            return Ok(r);
        }
        available.extend(i.drivers.iter().cloned());
        lookup |= i.uuid_lookup;
    }
    if !lookup {
        r.stage("Early userspace",false,"No selected initramfs can resolve root=UUID=. UUID lookup belongs to this supplied early-userspace model; the bare kernel does not resolve filesystem UUIDs here.");
        return Ok(r);
    }
    r.stage(
        "Early userspace",
        true,
        "Selected initramfs images match the kernel and provide filesystem UUID lookup.",
    );
    let Some(root) = inventory
        .roots
        .iter()
        .find(|v| v.uuid.eq_ignore_ascii_case(&o.root))
    else {
        r.stage("Root filesystem",false,"The root=UUID= value matches no supplied filesystem. Compare it with the filesystem inventory; changing title or version will not change the boot target.");
        return Ok(r);
    };
    if !root_available(root) {
        r.stage(
            "Root filesystem",
            false,
            format!("The root UUID {} is on a disk that is unavailable in this boot test.", root.uuid),
        );
        return Ok(r);
    }
    let missing = root
        .required_drivers
        .iter()
        .filter(|d| !available.contains(*d))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        r.stage("Root filesystem",false,format!("The selected early boot lacks {} needed by {}. Drivers must be built in or supplied by matching initramfs images before the real root can be mounted.",missing.join(", "),root.label));
        return Ok(r);
    }
    r.root = Some(root.uuid.to_ascii_lowercase());
    r.stage(
        "Root filesystem",
        true,
        format!(
            "Located {} ({}) with the supplied storage/filesystem drivers; initial mode {}.",
            root.label,
            root.uuid,
            if o.read_only {
                "read-only"
            } else {
                "read/write"
            }
        ),
    );
    if !root.init_present {
        r.stage("Init handoff",false,"That filesystem has no working supplied init program. Reaching a filesystem is not enough; check whether it is the intended system root.");
        return Ok(r);
    }
    r.target = Some(o.target);
    r.stage("Init handoff",true,"The supplied init program is available; the modeled handoff can request the selected target. No real program or services are started.");
    Ok(r)
}
