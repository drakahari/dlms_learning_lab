//! Literal persistent defaults and supplied-template generation, never shell execution.
use crate::{boot_entry as boot, grub_menu::Menu};
use boot::{Error, ErrorKind};
type Result<T> = std::result::Result<T, Error>;
pub const HELP: &str = "Only literal GRUB_DEFAULT, GRUB_TIMEOUT, GRUB_CMDLINE_LINUX and GRUB_CMDLINE_LINUX_DEFAULT assignments are supported here. Missing values use 0, 5 seconds and empty argument lists. Later assignments replace earlier ones. Single/double quotes, literal concatenation, ordinary comments and escapes are supported; expansions (including unquoted tilde), sourcing, shell commands and line continuations are not. Timeout is limited to -1 or 0–300 seconds in this simulator. Supplied ordered templates stand in for distribution discovery/scripts; they are not run. Generate changes only a virtual menu. Editing source settings does not regenerate it.";
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Defaults {
    pub selection: String,
    pub timeout: i32,
    pub common: Vec<String>,
    pub normal: Vec<String>,
}
impl Default for Defaults {
    fn default() -> Self {
        Self {
            selection: "0".into(),
            timeout: 5,
            common: vec![],
            normal: vec![],
        }
    }
}
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
fn assignment(line: &str) -> Result<Option<(String, String)>> {
    let s = line.trim_start();
    if s.is_empty() || s.starts_with('#') {
        return Ok(None);
    }
    let (key, rest) = s.split_once('=').ok_or_else(|| {
        error(
            ErrorKind::Syntax,
            "Use NAME=value with no spaces around =; this editor accepts settings, not commands.",
        )
    })?;
    if key.is_empty() || !key.bytes().all(|b| b.is_ascii_uppercase() || b == b'_') {
        return Err(error(
            ErrorKind::Unsupported,
            "Use one supported literal settings assignment per line; commands and exports are outside this subset.",
        ));
    }
    let mut value = String::new();
    let mut quote = None;
    let mut chars = rest.chars().peekable();
    let mut ended = false;
    while let Some(c) = chars.next() {
        if ended {
            if c == '#' {
                break;
            }
            if c.is_ascii_whitespace() {
                continue;
            }
            return Err(error(
                ErrorKind::Unsupported,
                "Extra words or commands follow a setting. Quote values containing spaces; nothing was executed.",
            ));
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
                continue;
            }
            if q == '"' && c == '\\' {
                let n = chars.next().ok_or_else(|| {
                    error(
                        ErrorKind::Syntax,
                        "Finish the escaped character and close the quote.",
                    )
                })?;
                if matches!(n, '$' | '`' | '"' | '\\') {
                    value.push(n);
                } else {
                    value.push('\\');
                    value.push(n);
                }
            } else if q == '"' && matches!(c, '$' | '`') {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Expansion is not supported in these literal settings. Write the intended value directly.",
                ));
            } else {
                value.push(c);
            }
        } else {
            match c {
                '\''|'"'=>quote=Some(c),
                '\\'=>value.push(chars.next().ok_or_else(||error(ErrorKind::Unsupported,"Line continuations are outside this settings subset; keep the assignment on one line."))?),
                ' '| '\t'=>ended=true,
                '$'|'`'|'~'|';'|'&'|'|'|'<'|'>'|'('|')'=>return Err(error(ErrorKind::Unsupported,"Expansion, redirection and shell commands are outside this settings subset; nothing was executed.")),
                _=>value.push(c),
            }
        }
    }
    if quote.is_some() {
        return Err(error(
            ErrorKind::Syntax,
            "Close the quoted setting on this line.",
        ));
    }
    Ok(Some((key.into(), value)))
}
fn arguments(value: &str) -> Result<Vec<String>> {
    // Values have undergone only the explicitly supported assignment quote removal.
    // Real template expansion is not evaluated; only ordinary literal argument words.
    let words: Vec<_> = value.split_ascii_whitespace().map(str::to_owned).collect();
    if words.len() > 16 {
        return Err(error(
            ErrorKind::Limit,
            "Use at most 16 kernel arguments per setting.",
        ));
    }
    for word in &words {
        if !matches!(word.as_str(), "ro" | "rw" | "quiet") && !word.starts_with("systemd.unit=") {
            return Err(error(
                ErrorKind::Unsupported,
                format!(
                    "Kernel argument {word:?} is outside this settings subset. The supplied templates own root UUIDs and image paths."
                ),
            ));
        }
        if let Some(target) = word.strip_prefix("systemd.unit=")
            && !matches!(target, "multi-user.target" | "rescue.target")
        {
            return Err(error(
                ErrorKind::Unsupported,
                "Only ordinary multi-user and rescue startup targets are modeled.",
            ));
        }
    }
    Ok(words)
}
impl Defaults {
    pub fn parse(source: &str) -> Result<Self> {
        if source.len() > boot::MAX_SOURCE || source.lines().count() > boot::MAX_LINES {
            return Err(error(
                ErrorKind::Limit,
                "Keep settings within 4 KiB and 48 lines.",
            ));
        }
        if source
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
        {
            return Err(error(
                ErrorKind::Syntax,
                "Use ordinary text and LF newlines; control characters are not accepted.",
            ));
        }
        let mut parsed = Self::default();
        for (index, line) in source.lines().enumerate() {
            let result = (|| {
                let Some((key, value)) = assignment(line)? else {
                    return Ok(());
                };
                match key.as_str() {
                    "GRUB_DEFAULT" => {
                        if value.is_empty() || value.len() > 80 || value == "saved" {
                            return Err(error(
                                ErrorKind::Unsupported,
                                "Select a supplied position, ID or title; saved environments and empty selectors are outside this model.",
                            ));
                        }
                        parsed.selection = value;
                    }
                    "GRUB_TIMEOUT" => {
                        parsed.timeout = value.parse().map_err(|_| {
                            error(
                                ErrorKind::Syntax,
                                "Timeout needs a whole number of seconds, or -1 to wait.",
                            )
                        })?;
                        if !(-1..=300).contains(&parsed.timeout) {
                            return Err(error(
                                ErrorKind::Limit,
                                "Timeout is bounded to -1 or 0–300 seconds here.",
                            ));
                        }
                    }
                    "GRUB_CMDLINE_LINUX" => parsed.common = arguments(&value)?,
                    "GRUB_CMDLINE_LINUX_DEFAULT" => parsed.normal = arguments(&value)?,
                    _ => {
                        return Err(error(
                            ErrorKind::Unsupported,
                            format!(
                                "Setting {key:?} is outside the four-key subset; no setting was silently ignored."
                            ),
                        ));
                    }
                }
                Ok(())
            })();
            if let Err(mut e) = result {
                e.message = format!("Line {}: {}", index + 1, e.message);
                return Err(e);
            }
        }
        Ok(parsed)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Generated {
    pub timeout: i32,
    pub menu: Menu,
    /// Only the flat menu accepted by the existing literal-menu trace.
    pub menu_source: String,
    /// Complete read-only generated presentation, including the modeled timer.
    pub text: String,
}
fn quoted(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
pub fn generate(source: &str, templates: &str, normal_ids: &[String]) -> Result<Generated> {
    let settings = Defaults::parse(source)?;
    let mut menu = Menu::parse(templates)?;
    if normal_ids.is_empty()
        || normal_ids.len() > 4
        || normal_ids
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != normal_ids.len()
        || normal_ids
            .iter()
            .any(|id| !menu.items.iter().any(|e| &e.id == id))
    {
        return Err(error(
            ErrorKind::Syntax,
            "Supplied templates must identify unique normal entries; other entries are recovery choices.",
        ));
    }
    menu.default = settings.selection;
    for item in &mut menu.items {
        item.entry.options.extend(settings.common.iter().cloned());
        if normal_ids.contains(&item.id) {
            item.entry.options.extend(settings.normal.iter().cloned());
        }
        if item.entry.options.len() > 16 {
            return Err(error(
                ErrorKind::Limit,
                "The combined template and settings exceed 16 kernel arguments.",
            ));
        }
        // Honest diagnostic for unsupported ambiguous options; same core as menu lessons.
        boot::Options::parse(&item.entry.options)?;
    }
    let mut rendered = format!("set default={}\n", quoted(&menu.default));
    for item in &menu.items {
        rendered.push_str(&format!(
            "menuentry {} --id={} {{\n  linux {} {}\n",
            quoted(&item.title),
            quoted(&item.id),
            quoted(&item.entry.linux),
            item.entry
                .options
                .iter()
                .map(|v| quoted(v))
                .collect::<Vec<_>>()
                .join(" ")
        ));
        if !item.entry.initrd.is_empty() {
            rendered.push_str(&format!(
                "  initrd {}\n",
                item.entry
                    .initrd
                    .iter()
                    .map(|v| quoted(v))
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }
        rendered.push_str("}\n");
    }
    // Validate both literal selectors and the generated menu by the same real parser.
    let observed = Menu::parse(&rendered)?;
    if observed != menu {
        return Err(error(
            ErrorKind::Syntax,
            "Generated menu did not preserve its typed entries.",
        ));
    }
    Ok(Generated {
        timeout: settings.timeout,
        text: format!(
            "# Generated from supplied templates; do not edit this output.\nset timeout={}\n{rendered}",
            settings.timeout
        ),
        menu,
        menu_source: rendered,
    })
}
