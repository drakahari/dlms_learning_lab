//! Bounded standalone-share policy parsing, never host Samba.
//! Configuration checks do not prove service or filesystem availability.
use std::collections::BTreeMap;

pub const MAX_CONFIG: usize = 16 * 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Syntax,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: Kind,
    pub line: usize,
    pub detail: String,
}
fn fault(kind: Kind, line: usize, detail: impl Into<String>) -> Diagnostic {
    Diagnostic {
        kind,
        line,
        detail: detail.into(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Principal {
    User(String),
    Group(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Share {
    pub path: String,
    pub read_only: bool,
    pub guest_ok: bool,
    pub valid_users: Vec<Principal>,
    pub invalid_users: Vec<Principal>,
    pub read_list: Vec<Principal>,
    pub write_list: Vec<Principal>,
}
impl Default for Share {
    fn default() -> Self {
        Self {
            path: String::new(),
            read_only: true,
            guest_ok: false,
            valid_users: vec![],
            invalid_users: vec![],
            read_list: vec![],
            write_list: vec![],
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Configuration {
    pub shares: BTreeMap<String, Share>,
}
fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn boolean(value: &str, line: usize) -> Result<bool, Diagnostic> {
    match value.to_ascii_lowercase().as_str() {
        "yes" | "true" | "1" => Ok(true),
        "no" | "false" | "0" => Ok(false),
        _ => Err(fault(
            Kind::Unsupported,
            line,
            "Supported booleans are yes/no, true/false or 1/0.",
        )),
    }
}
fn principals(value: &str, line: usize) -> Result<Vec<Principal>, Diagnostic> {
    let mut result = vec![];
    for atom in value
        .split(|c: char| c.is_ascii_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
    {
        let (group, text) = atom.strip_prefix('@').map_or((false, atom), |s| (true, s));
        if !name(text) {
            return Err(fault(
                Kind::Unsupported,
                line,
                "Use bare canonical user names or @Unix-group names; quoted, domain, NIS and substitution forms are outside this simulator.",
            ));
        }
        if result.len() == 16 {
            return Err(fault(
                Kind::Limit,
                line,
                "A policy list supports at most16names.",
            ));
        }
        let p = if group {
            Principal::Group(text.into())
        } else {
            Principal::User(text.into())
        };
        if !result.contains(&p) {
            result.push(p)
        }
    }
    result.sort();
    Ok(result)
}
fn virtual_absolute(value: &str) -> bool {
    value.starts_with('/')
        && value.len() <= 512
        && (value == "/" || !value.ends_with('/'))
        && !value.chars().any(char::is_control)
        && (value == "/"
            || value
                .split('/')
                .skip(1)
                .all(|p| !p.is_empty() && p != "." && p != ".."))
}
/// This context is an already supplied standalone server; no defaults for AD or
/// protocol authentication are inferred. Missing paths may parse, then fail the
/// actual virtual access trial: internal config correctness is not availability.
pub fn parse(raw: &str) -> Result<Configuration, Diagnostic> {
    if raw.len() > MAX_CONFIG {
        return Err(fault(Kind::Limit, 0, "Configuration exceeds16KiB."));
    }
    if raw.contains('\0') {
        return Err(fault(Kind::Syntax, 0, "Remove the NUL character."));
    }
    let mut config = Configuration {
        shares: BTreeMap::new(),
    };
    let mut section: Option<String> = None;
    let mut saw_global = false;
    for (n, line) in raw.lines().enumerate() {
        let n = n + 1;
        if n > 128 || line.len() > 512 {
            return Err(fault(
                Kind::Limit,
                n,
                "Use at most128lines and512bytes per line.",
            ));
        }
        let line = line.trim();
        if line.is_empty() || line.starts_with(['#', ';']) {
            continue;
        }
        if line.ends_with('\\') {
            return Err(fault(
                Kind::Unsupported,
                n,
                "Continued lines are outside this editor's supported subset.",
            ));
        }
        if line.starts_with('[') {
            let value = line
                .strip_prefix('[')
                .and_then(|s| s.strip_suffix(']'))
                .ok_or_else(|| fault(Kind::Syntax, n, "Close the section name with ]."))?
                .trim();
            if !name(value) {
                return Err(fault(
                    Kind::Unsupported,
                    n,
                    "Use a supplied simple share name or [global].",
                ));
            }
            let key = value.to_ascii_lowercase();
            if key == "global" {
                if saw_global {
                    return Err(fault(
                        Kind::Unsupported,
                        n,
                        "Combine the global settings into one [global] section.",
                    ));
                }
                saw_global = true;
            } else {
                if matches!(key.as_str(), "homes" | "printers") {
                    return Err(fault(
                        Kind::Unsupported,
                        n,
                        "Automatic home and printer shares are not simulated.",
                    ));
                }
                if config.shares.contains_key(&key) {
                    return Err(fault(
                        Kind::Unsupported,
                        n,
                        "Combine repeated share sections before checking.",
                    ));
                }
                if config.shares.len() == 8 {
                    return Err(fault(Kind::Limit, n, "At most8shares are supported."));
                }
                config.shares.insert(key.clone(), Share::default());
            }
            section = Some(key);
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| fault(Kind::Syntax, n, "Write a setting as name = value."))?;
        let key = key
            .split_ascii_whitespace()
            .collect::<String>()
            .to_ascii_lowercase();
        let value = value.trim();
        let section = section.as_ref().ok_or_else(|| {
            fault(
                Kind::Syntax,
                n,
                "Put settings under [global] or a share section.",
            )
        })?;
        if value.contains(['%', '\"']) {
            return Err(fault(
                Kind::Unsupported,
                n,
                "Substitutions and quoted values are not simulated; use the documented plain values.",
            ));
        }
        if section == "global" {
            let supported = match key.as_str() {
                "security" => value.eq_ignore_ascii_case("user"),
                "serverrole" => value.eq_ignore_ascii_case("standalone server"),
                "maptoguest" => value.eq_ignore_ascii_case("never"),
                _ => false,
            };
            if !supported {
                return Err(fault(
                    Kind::Unsupported,
                    n,
                    "The supplied server supports only security=user, server role=standalone server and map to guest=Never global settings.",
                ));
            }
            continue;
        }
        let share = config.shares.get_mut(section).unwrap();
        match key.as_str() {
            "path" => {
                if !virtual_absolute(value) {
                    return Err(fault(
                        Kind::Unsupported,
                        n,
                        "Use a canonical absolute simulated path; dot segments and relative paths are not supported.",
                    ));
                }
                share.path = value.into();
            }
            "readonly" => share.read_only = boolean(value, n)?,
            "writeable" | "writable" => share.read_only = !boolean(value, n)?,
            "guestok" => share.guest_ok = boolean(value, n)?,
            "validusers" => share.valid_users = principals(value, n)?,
            "invalidusers" => share.invalid_users = principals(value, n)?,
            "readlist" => share.read_list = principals(value, n)?,
            "writelist" => share.write_list = principals(value, n)?,
            _ => {
                return Err(fault(
                    Kind::Unsupported,
                    n,
                    format!("Setting {key} is outside the supported standalone-share subset."),
                ));
            }
        }
    }
    if config.shares.is_empty() {
        return Err(fault(Kind::Syntax, 0, "Add a named share section."));
    }
    for (name, share) in &config.shares {
        if share.path.is_empty() {
            return Err(fault(
                Kind::Unsupported,
                0,
                format!(
                    "Supply an explicit virtual path in [{name}]; implicit share paths are not simulated."
                ),
            ));
        }
    }
    Ok(config)
}
