//! Closed local-file Sun-map subset. Unsupported AutoFS syntax is diagnosed,
//! never interpreted as a program, host file, network source or shell command.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_TEXT: usize = 4096;
pub const MAX_LINES: usize = 64;
pub const MAX_ROOTS: usize = 2;
pub const MAX_ENTRIES: usize = 8;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    Syntax,
    Unsupported,
    Limit,
    Context,
    Lookup,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: Option<usize>,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(n) = self.line {
            write!(f, "Line {n}: ")?;
        }
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn err(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        line: None,
        message: message.into(),
    }
}
fn at(mut e: Error, line: usize) -> Error {
    e.line = Some(line);
    e
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Style {
    Direct,
    Indirect,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Root {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub point: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub file: String,
    pub timeout: u16,
}
impl Root {
    pub fn style(&self) -> Style {
        if self.point == "/-" {
            Style::Direct
        } else {
            Style::Indirect
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub key: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub target: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub device: String,
    pub read_only: bool,
}
pub fn text(value: &str) -> Result<()> {
    if value.len() > MAX_TEXT || value.lines().count() > MAX_LINES {
        return Err(err(
            ErrorKind::Limit,
            "Use at most 4096 bytes and 64 lines per supplied map file",
        ));
    }
    if !value.is_ascii()
        || value
            .bytes()
            .any(|b| b.is_ascii_control() && !matches!(b, b'\n' | b'\r' | b'\t'))
    {
        return Err(err(
            ErrorKind::Unsupported,
            "This map subset supports ordinary ASCII text and whitespace; no control characters",
        ));
    }
    Ok(())
}
pub fn absolute(value: &str) -> bool {
    value.starts_with('/')
        && value.len() > 1
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'_' | b'-' | b'.'))
        && !value
            .split('/')
            .skip(1)
            .any(|p| p.is_empty() || matches!(p, "." | ".."))
}
fn component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && !matches!(value, "." | "..")
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}
fn ordinary(line: &str, n: usize) -> Result<Vec<&str>> {
    if line.contains(['\\', '"', '\'', '$', '&', '*', '`', '|', ';', '(', ')'])
        || line.starts_with('+')
    {
        return Err(at(
            err(
                ErrorKind::Unsupported,
                "Includes, wildcards, substitutions, escaping, quoted fields, multi-mounts and program/shell maps are outside this local-file subset; nothing executes",
            ),
            n,
        ));
    }
    if line.contains('#') {
        return Err(at(
            err(
                ErrorKind::Unsupported,
                "Use whole-line comments in this subset; inline comment parsing is not implemented",
            ),
            n,
        ));
    }
    Ok(line.split_ascii_whitespace().collect())
}
/// Parse a saved master map using only a supplied virtual file roster. Empty is
/// a valid map managing no points. Defaults/duplicate ambiguity are not guessed.
pub fn master(value: &str, files: &[String]) -> Result<Vec<Root>> {
    text(value)?;
    if files.is_empty()
        || files.len() > 3
        || files
            .iter()
            .any(|p| !absolute(p) || p == "/etc/auto.master")
    {
        return Err(err(
            ErrorKind::Context,
            "Supply 1–3 distinct virtual map filenames, separate from /etc/auto.master",
        ));
    }
    if files.iter().collect::<BTreeSet<_>>().len() != files.len() {
        return Err(err(
            ErrorKind::Context,
            "Supply each virtual map filename once",
        ));
    }
    let mut roots = Vec::<Root>::new();
    for (n, line) in value.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if roots.len() == MAX_ROOTS {
            return Err(at(
                err(
                    ErrorKind::Limit,
                    "This exercise supports at most two master-map entries",
                ),
                n + 1,
            ));
        }
        let fields = ordinary(line, n + 1)?;
        let [point, file, option] = fields.as_slice() else {
            return Err(at(
                err(
                    ErrorKind::Unsupported,
                    "Use mountpoint map-file --timeout=SECONDS. Implicit daemon defaults and other master options are outside this exercise",
                ),
                n + 1,
            ));
        };
        if !absolute(point) {
            return Err(at(
                err(
                    ErrorKind::Syntax,
                    "Use an absolute directory for an indirect map, or /- for a direct map",
                ),
                n + 1,
            ));
        }
        let file = file.strip_prefix("file:").unwrap_or(file);
        if !absolute(file) || !files.iter().any(|p| p == file) {
            return Err(at(
                err(
                    ErrorKind::Unsupported,
                    "Choose a supplied plain file or file: map. No program, network, NSS or host file is opened",
                ),
                n + 1,
            ));
        }
        let Some(number) = option.strip_prefix("--timeout=") else {
            return Err(at(
                err(
                    ErrorKind::Unsupported,
                    "Only an explicit --timeout=SECONDS master option is supported",
                ),
                n + 1,
            ));
        };
        if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
            return Err(at(
                err(
                    ErrorKind::Syntax,
                    "Idle timeout needs a whole number of seconds",
                ),
                n + 1,
            ));
        }
        let timeout=number.parse::<u16>().ok().filter(|n|(1..=3600).contains(n)).ok_or_else(||at(err(ErrorKind::Unsupported,"Use an explicit timeout from 1 through 3600 seconds. Zero/disabled expiry and other daemon defaults are not modeled"),n+1))?;
        let root = Root {
            point: (*point).into(),
            file: file.into(),
            timeout,
        };
        if roots
            .iter()
            .any(|r| r.file == root.file || r.point == root.point && r.point != "/-")
        {
            return Err(at(
                err(
                    ErrorKind::Unsupported,
                    "Duplicate/reused maps are outside this subset. Real AutoFS ignores later duplicate indirect roots; this model rejects the ambiguity instead",
                ),
                n + 1,
            ));
        }
        if root.style() == Style::Indirect
            && roots
                .iter()
                .filter(|r| r.style() == Style::Indirect)
                .any(|r| {
                    root.point.starts_with(&format!("{}/", r.point))
                        || r.point.starts_with(&format!("{}/", root.point))
                })
        {
            return Err(at(
                err(
                    ErrorKind::Unsupported,
                    "Nested managed roots are outside this exercise",
                ),
                n + 1,
            ));
        }
        roots.push(root);
    }
    roots.sort_by(|a, b| (&a.point, &a.file).cmp(&(&b.point, &b.file)));
    Ok(roots)
}
/// Parse a local-device map. Source existence/type/availability is evaluated by
/// the actual virtual mount model later; parsing never claims a mounted result.
pub fn entries(value: &str, root: &Root, points: &BTreeSet<String>) -> Result<Vec<Entry>> {
    text(value)?;
    if !absolute(&root.point)
        || !absolute(&root.file)
        || !(1..=3600).contains(&root.timeout)
        || points.is_empty()
        || points.len() > 4
        || points.iter().any(|p| !absolute(p) || p == "/-")
    {
        return Err(err(
            ErrorKind::Context,
            "Use a validated root and 1–4 supplied virtual mountpoints",
        ));
    }
    let mut entries = Vec::<Entry>::new();
    let mut keys = BTreeSet::new();
    for (n, line) in value.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if entries.len() == MAX_ENTRIES {
            return Err(at(
                err(
                    ErrorKind::Limit,
                    "Use at most eight entries per supplied map",
                ),
                n + 1,
            ));
        }
        let fields = ordinary(line, n + 1)?;
        let [key, options, location] = fields.as_slice() else {
            return Err(at(
                err(
                    ErrorKind::Unsupported,
                    "Use key -fstype=ext4[,ro|rw] :/dev/DEVICE. Default NFS, multiple locations and additional fields are outside this exercise",
                ),
                n + 1,
            ));
        };
        let target = match root.style() {
            Style::Direct => {
                if !absolute(key) || *key == "/-" {
                    return Err(at(
                        err(
                            ErrorKind::Syntax,
                            "A direct map uses the full absolute mountpoint as its key; /- belongs only in the master map",
                        ),
                        n + 1,
                    ));
                }
                (*key).to_owned()
            }
            Style::Indirect => {
                if !component(key) {
                    return Err(at(
                        err(
                            ErrorKind::Syntax,
                            "An indirect-map key is one directory name under the master's directory, without a slash or wildcard",
                        ),
                        n + 1,
                    ));
                }
                format!("{}/{key}", root.point)
            }
        };
        if !points.contains(&target) {
            return Err(at(
                err(
                    ErrorKind::Unsupported,
                    format!(
                        "{target} is outside the supplied virtual mountpoints; check the master directory and key together"
                    ),
                ),
                n + 1,
            ));
        }
        if !keys.insert((*key).to_owned()) {
            return Err(at(
                err(
                    ErrorKind::Unsupported,
                    "Duplicate map keys are not resolved by guessing precedence in this subset",
                ),
                n + 1,
            ));
        }
        let Some(options) = options.strip_prefix('-') else {
            return Err(at(
                err(
                    ErrorKind::Syntax,
                    "Put map options after a leading dash, for example -fstype=ext4,ro",
                ),
                n + 1,
            ));
        };
        if options.len() > 80 {
            return Err(at(
                err(ErrorKind::Limit, "Map options exceed the supported length"),
                n + 1,
            ));
        }
        let mut fs = false;
        let mut access = None;
        let mut defaults = false;
        for option in options.split(',') {
            match option {
                "fstype=ext4" if !fs => fs = true,
                "ro" | "rw" if access.is_none() => access = Some(option == "ro"),
                "defaults" if !defaults => defaults = true,
                _ => {
                    return Err(at(
                        err(
                            ErrorKind::Unsupported,
                            "Supported unique map options are fstype=ext4, one ro/rw choice, and defaults. No other filesystem/mount options are inferred",
                        ),
                        n + 1,
                    ));
                }
            }
        }
        if !fs {
            return Err(at(
                err(
                    ErrorKind::Unsupported,
                    "Local devices need explicit -fstype=ext4 in this exercise; implicit NFS behavior is not modeled",
                ),
                n + 1,
            ));
        }
        let Some(device) = location.strip_prefix(':') else {
            return Err(at(
                err(
                    ErrorKind::Syntax,
                    "Put a colon before a local /dev path, such as :/dev/DEVICE; this exercise does not contact NFS servers",
                ),
                n + 1,
            ));
        };
        if !absolute(device) || !device.starts_with("/dev/") {
            return Err(at(
                err(
                    ErrorKind::Unsupported,
                    "Choose a supplied local /dev source. UUID tags, remote/program locations and host path lookup are outside this map subset",
                ),
                n + 1,
            ));
        }
        entries.push(Entry {
            key: (*key).into(),
            target,
            device: device.into(),
            read_only: access.unwrap_or(false),
        });
    }
    Ok(entries)
}
