//! Pure offline authentication-filter matching; no files/processes/network.
//! Narrow capability; no lesson is registered implicitly.
use crate::ere;
use std::net::Ipv4Addr;

pub const MAX_FILTER: usize = 2048;
pub const MAX_LINES: usize = 64;
pub const MAX_LINE: usize = 512;
pub const MAX_PATTERN: usize = 192;
const MAX_WORK: usize = 250_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Syntax,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub kind: Kind,
    pub detail: String,
}
type Result<T> = std::result::Result<T, Fault>;
fn err<T>(kind: Kind, detail: impl Into<String>) -> Result<T> {
    Err(Fault {
        kind,
        detail: detail.into(),
    })
}
fn regex_fault(e: ere::Error) -> Fault {
    Fault {
        kind: match e.kind {
            ere::ErrorKind::Syntax => Kind::Syntax,
            ere::ErrorKind::Unsupported => Kind::Unsupported,
            ere::ErrorKind::Limit => Kind::Limit,
        },
        detail: e.to_string(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    pub fail: String,
    pub ignore: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    Matched,
    Ignored,
    Missed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub line: usize,
    pub text: String,
    pub address: Option<Ipv4Addr>,
    pub disposition: Disposition,
}

/// Common ASCII regular syntax, not arbitrary Python/ERE. Refuse unsupported
/// syntax before compilation, including constructs with different semantics.
fn syntax(pattern: &str, host: bool) -> Result<()> {
    if pattern.len() > MAX_PATTERN {
        return err(Kind::Limit, "Filter pattern exceeds 192 bytes.");
    }
    if pattern.is_empty() || !pattern.is_ascii() || pattern.bytes().any(|b| b.is_ascii_control()) {
        return err(
            Kind::Unsupported,
            "Use a nonempty single-line ASCII pattern.",
        );
    }
    if pattern.matches("<HOST>").count() != usize::from(host) {
        return err(
            Kind::Syntax,
            if host {
                "failregex needs exactly one <HOST> address tag."
            } else {
                "ignoreregex does not support address tags in this activity."
            },
        );
    }
    if host {
        let at = pattern.find("<HOST>").unwrap();
        let bytes = pattern.as_bytes();
        let before = at.checked_sub(1).and_then(|i| bytes.get(i)).copied();
        let after = bytes.get(at + 6).copied();
        if !before.is_none_or(|b| b" =:^".contains(&b)) || !after.is_none_or(|b| b" $".contains(&b))
        {
            return err(
                Kind::Unsupported,
                "Separate <HOST> from other fields with literal spaces or an =/: prefix; an address must be a complete token.",
            );
        }
        let mut class = false;
        let mut escaped = false;
        let mut depth = 0usize;
        for (i, b) in pattern.bytes().enumerate() {
            if i == at && (class || escaped || depth != 0) {
                return err(
                    Kind::Unsupported,
                    "Put the required <HOST> tag outside groups and bracket classes; do not escape it.",
                );
            }
            if escaped {
                escaped = false;
                continue;
            }
            if b == b'\\' {
                escaped = true;
                continue;
            }
            if b == b'[' {
                class = true;
            } else if b == b']' {
                class = false;
            } else if !class {
                match b {
                    b'(' => depth += 1,
                    b')' => depth = depth.saturating_sub(1),
                    b'|' if depth == 0 => {
                        return err(
                            Kind::Unsupported,
                            "Keep alternatives inside the non-address fields. Every matching path must require <HOST>.",
                        );
                    }
                    _ => {}
                }
            }
        }
    }
    let plain = pattern.replace("<HOST>", "203\\.0\\.113\\.9");
    let bytes = plain.as_bytes();
    let mut i = 0;
    let mut in_class = false;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' {
            i += 1;
            if i == bytes.len() {
                return err(
                    Kind::Syntax,
                    "A final backslash needs a literal character to escape.",
                );
            }
            if !b".[](){}*+?^$|\\-".contains(&bytes[i]) {
                return err(
                    Kind::Unsupported,
                    "Only escaped literal punctuation is supported; use explicit ASCII classes instead of shorthand escapes or backreferences.",
                );
            }
        } else if b == b'[' {
            if in_class {
                return err(
                    Kind::Unsupported,
                    "Nested/POSIX bracket classes are outside this common ASCII subset.",
                );
            }
            in_class = true;
        } else if b == b']' {
            in_class = false;
        } else if !in_class && (b == b'<' || b == b'>') {
            return err(
                Kind::Unsupported,
                "Only the <HOST> tag is supported; other tags are not evaluated.",
            );
        } else if !in_class && b == b'(' && bytes.get(i + 1) == Some(&b'?') {
            return err(
                Kind::Unsupported,
                "Lookarounds, inline flags and special groups are not supported.",
            );
        } else if !in_class
            && b"*+?}".contains(&b)
            && bytes.get(i + 1).is_some_and(|n| b"*+?".contains(n))
        {
            return err(
                Kind::Unsupported,
                "Lazy, possessive and repeated quantifiers are not supported.",
            );
        }
        i += 1;
    }
    ere::Regex::compile(&plain).map_err(regex_fault)?;
    Ok(())
}
impl Filter {
    pub fn parse(text: &str) -> Result<Self> {
        if text.len() > MAX_FILTER {
            return err(Kind::Limit, "Filter text exceeds 2 KiB; no test was run.");
        }
        if !text.is_ascii() || text.bytes().any(|b| b == 0 || b == b'\r') {
            return err(
                Kind::Unsupported,
                "Use ASCII filter text with ordinary line breaks.",
            );
        }
        let mut section = false;
        let mut fail = None;
        let mut ignore = None;
        for (n, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if line == "[Definition]" {
                if section {
                    return err(
                        Kind::Syntax,
                        "The filter has a repeated [Definition] section.",
                    );
                }
                section = true;
                continue;
            }
            if line.starts_with('[') {
                return err(
                    Kind::Unsupported,
                    format!(
                        "Line{}: only [Definition] is supported; includes and initialization are not evaluated.",
                        n + 1
                    ),
                );
            }
            if !section {
                return err(
                    Kind::Syntax,
                    format!("Line{}: put filter settings below [Definition].", n + 1),
                );
            }
            let Some((name, value)) = line.split_once('=') else {
                return err(
                    Kind::Unsupported,
                    format!(
                        "Line{}: use one name=value setting per line; continuation lines are not supported.",
                        n + 1
                    ),
                );
            };
            let value = value.split_once("; ").map_or(value, |x| x.0).trim();
            let slot = match name.trim() {
                "failregex" => &mut fail,
                "ignoreregex" => &mut ignore,
                _ => {
                    return err(
                        Kind::Unsupported,
                        format!(
                            "Line{}: this offline activity supports failregex and ignoreregex only; no other setting was applied.",
                            n + 1
                        ),
                    );
                }
            };
            if slot.is_some() {
                return err(
                    Kind::Syntax,
                    format!(
                        "Line{}: combine alternatives into one pattern instead of repeating a setting.",
                        n + 1
                    ),
                );
            }
            *slot = Some(value.to_owned());
        }
        let fail = fail.ok_or_else(|| Fault {
            kind: Kind::Syntax,
            detail: "Add [Definition] and a failregex setting before testing.".into(),
        })?;
        syntax(&fail, true)?;
        // An empty ignoreregex deliberately means no ignored messages.
        let ignore = ignore.filter(|s| !s.is_empty());
        if let Some(pattern) = &ignore {
            syntax(pattern, false)?;
        }
        Ok(Self { fail, ignore })
    }
}
/// Source logs admit at most one canonical IPv4 token per line. This restriction
/// makes boolean macro matching unambiguous; it is never general capture support.
fn address(line: &str) -> Result<Option<Ipv4Addr>> {
    let mut found = None;
    for token in line.split(|c: char| !c.is_ascii_digit() && c != '.') {
        if !token.contains('.') {
            continue;
        }
        if let Ok(ip) = token.parse::<Ipv4Addr>() {
            if found.is_some() {
                return err(
                    Kind::Unsupported,
                    "This offline activity supports one IPv4 token per log line; multiple-address capture is not modeled.",
                );
            }
            found = Some(ip);
        }
    }
    Ok(found)
}
fn matched(pattern: &str, line: &str, budget: &mut usize) -> Result<bool> {
    let regex = ere::Regex::compile(pattern).map_err(regex_fault)?;
    *budget=budget.checked_sub(regex.compile_work()).ok_or_else(||Fault{kind:Kind::Limit,detail:"The whole filter test exhausted its bounded work allowance; no partial result was saved.".into()})?;
    let mut steps = ere::Budget::new(*budget).map_err(regex_fault)?;
    let result = regex
        .matcher()
        .map_err(regex_fault)?
        .matches_line(line, &mut steps)
        .map_err(regex_fault)?;
    *budget = steps.remaining();
    Ok(result)
}
pub fn inspect(filter: &Filter, lines: &[String]) -> Result<Vec<Row>> {
    // Public structs cannot bypass input validation after construction.
    syntax(&filter.fail, true)?;
    if let Some(s) = &filter.ignore {
        syntax(s, false)?;
    }
    if lines.len() > MAX_LINES {
        return err(Kind::Limit, "This test supports at most 64 log lines.");
    }
    let mut addresses = Vec::with_capacity(lines.len());
    for line in lines {
        if line.len() > MAX_LINE {
            return err(Kind::Limit, "A log line exceeds 512 bytes.");
        }
        if !line.is_ascii() || line.bytes().any(|b| b.is_ascii_control()) {
            return err(
                Kind::Unsupported,
                "Use single-line ASCII application messages; multiline/Unicode/date processing is not modeled.",
            );
        }
        addresses.push(address(line)?);
    }
    let mut budget = MAX_WORK;
    let mut rows = Vec::with_capacity(lines.len());
    for (n, (line, ip)) in lines.iter().zip(addresses).enumerate() {
        let disposition = if filter
            .ignore
            .as_ref()
            .map(|s| matched(s, line, &mut budget))
            .transpose()?
            .unwrap_or(false)
        {
            Disposition::Ignored
        } else if let Some(ip) = ip {
            let literal = ip.to_string().replace('.', "\\.");
            let expanded = filter.fail.replace("<HOST>", &literal);
            if matched(&expanded, line, &mut budget)? {
                Disposition::Matched
            } else {
                Disposition::Missed
            }
        } else {
            Disposition::Missed
        };
        rows.push(Row {
            line: n + 1,
            text: line.clone(),
            address: ip,
            disposition,
        });
    }
    Ok(rows)
}
