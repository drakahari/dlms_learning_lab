//! Bounded deb822 binary-source selection excerpt, not APT transport or trust.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_INPUT: usize = 8192;
pub const MAX_LINES: usize = 128;
pub const MAX_STANZAS: usize = 4;
pub const MAX_KEYS: usize = 24;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Syntax,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub code: Code,
    pub line: usize,
    pub message: &'static str,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 {
            f.write_str(self.message)
        } else {
            write!(f, "Line {}: {}", self.line, self.message)
        }
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn fail<T>(code: Code, line: usize, message: &'static str) -> Result<T> {
    Err(Error {
        code,
        line,
        message,
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Stanza {
    pub enabled: bool,
    pub uris: Vec<String>,
    pub suites: Vec<String>,
    pub components: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    #[serde(deserialize_with = "crate::debian_packages_workspace::string::<_,160>")]
    pub uri: String,
    #[serde(deserialize_with = "crate::debian_packages_workspace::string::<_,40>")]
    pub suite: String,
    #[serde(deserialize_with = "crate::debian_packages_workspace::string::<_,40>")]
    pub component: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Selection {
    pub keys: Vec<Key>,
    pub repeated_targets: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Field {
    Types,
    Uris,
    Suites,
    Components,
    Enabled,
}
fn word(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 40
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
        && s != "."
        && s != ".."
}
/// URI is only an offline identity. Never opened. Deliberately narrow HTTPS form.
pub fn uri(s: &str) -> Result<String> {
    if s.len() > 160 {
        return fail(Code::Limit, 0, "Source address exceeds 160 bytes");
    }
    let Some(rest) = s.strip_prefix("https://") else {
        return fail(
            Code::Unsupported,
            0,
            "Use the supplied ordinary HTTPS source label; other schemes are outside this selection excerpt",
        );
    };
    if !s.is_ascii() || s.chars().any(char::is_control) || s.contains(['@', '?', '#', '%', '\\']) {
        return fail(
            Code::Unsupported,
            0,
            "Credentials, queries, fragments, escapes and non-ASCII URI forms are not modeled",
        );
    }
    let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
    if host.is_empty()
        || host.len() > 100
        || !host.split('.').all(|p| {
            !p.is_empty()
                && p.len() <= 63
                && p.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                && !p.starts_with('-')
                && !p.ends_with('-')
        })
    {
        return fail(
            Code::Unsupported,
            0,
            "Use a supplied lowercase host name without a port or address-literal selector",
        );
    }
    if !path
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"/._+-~".contains(&b))
        || path.split('/').any(|p| p == "." || p == "..")
    {
        return fail(
            Code::Unsupported,
            0,
            "Use the supplied ordinary repository path without traversal or escape syntax",
        );
    }
    if s.ends_with('/') {
        Ok(s.into())
    } else {
        Ok(format!("{s}/"))
    }
}
impl Key {
    pub fn validate(&self) -> Result<()> {
        if uri(&self.uri)? != self.uri {
            return fail(
                Code::Syntax,
                0,
                "Stored source keys must use the canonical trailing URI slash",
            );
        }
        if !word(&self.suite) || !word(&self.component) {
            return fail(
                Code::Unsupported,
                0,
                "Stored source key needs an ordinary suite and component",
            );
        }
        Ok(())
    }
}
impl Stanza {
    pub fn validate(&self) -> Result<()> {
        if self.uris.len() > 2
            || self.suites.len() > 2
            || self.components.len() > 3
            || self
                .uris
                .iter()
                .chain(&self.suites)
                .chain(&self.components)
                .any(|s| s.len() > 160)
        {
            return fail(
                Code::Limit,
                0,
                "Source selection field/count bounds exceeded",
            );
        }
        if self
            .uris
            .iter()
            .chain(&self.suites)
            .chain(&self.components)
            .any(|s| !s.is_ascii() || s.chars().any(char::is_control))
        {
            return fail(
                Code::Unsupported,
                0,
                "Only ordinary ASCII source selection values are modeled",
            );
        }
        if self.enabled {
            if self.uris.is_empty() || self.suites.is_empty() || self.components.is_empty() {
                return fail(
                    Code::Syntax,
                    0,
                    "Enabled deb sources need URIs, Suites and Components",
                );
            }
            for u in &self.uris {
                uri(u)?;
            }
            if !self.suites.iter().chain(&self.components).all(|s| word(s)) {
                return fail(
                    Code::Unsupported,
                    0,
                    "Use ordinary suite/component names; flat repositories and suite expressions are outside this excerpt",
                );
            }
        }
        Ok(())
    }
}
fn values(
    map: &BTreeMap<Field, String>,
    field: Field,
    max: usize,
    line: usize,
) -> Result<Vec<String>> {
    let Some(s) = map.get(&field) else {
        return Ok(vec![]);
    };
    let mut values = vec![];
    for value in s.split_ascii_whitespace() {
        if values.len() >= max {
            return fail(
                Code::Limit,
                line,
                "Too many source field values for this activity",
            );
        }
        values.push(value.into());
    }
    Ok(values)
}
fn finish(fields: &mut BTreeMap<Field, String>, out: &mut Vec<Stanza>, line: usize) -> Result<()> {
    if fields.is_empty() {
        return Ok(());
    }
    if out.len() >= MAX_STANZAS {
        return fail(
            Code::Limit,
            line,
            "At most four source stanzas are supplied in this activity",
        );
    }
    let enabled = match fields.get(&Field::Enabled).map(String::as_str) {
        None => true,
        Some(v) if v.eq_ignore_ascii_case("yes") => true,
        Some(v) if v.eq_ignore_ascii_case("no") => false,
        _ => {
            return fail(
                Code::Unsupported,
                line,
                "Enabled supports yes or no in this excerpt",
            );
        }
    };
    if fields.get(&Field::Types).is_some_and(|v| v != "deb") {
        return fail(
            Code::Unsupported,
            line,
            "Only Types: deb is modeled; source-code and mixed-type acquisition are outside this activity",
        );
    }
    if enabled && !fields.contains_key(&Field::Types) {
        return fail(Code::Syntax, line, "An enabled stanza needs Types: deb");
    }
    let s = Stanza {
        enabled,
        uris: values(fields, Field::Uris, 2, line)?,
        suites: values(fields, Field::Suites, 2, line)?,
        components: values(fields, Field::Components, 3, line)?,
    };
    s.validate().map_err(|e| Error { line, ..e })?;
    out.push(s);
    fields.clear();
    Ok(())
}
pub fn parse(input: &str) -> Result<Vec<Stanza>> {
    if input.len() > MAX_INPUT {
        return fail(Code::Limit, 0, "Source selection excerpt exceeds 8 KiB");
    }
    let mut out = vec![];
    let mut fields = BTreeMap::<Field, String>::new();
    let mut last = None;
    let mut count = 0;
    for (i, line) in input.lines().enumerate() {
        let n = i + 1;
        count = n;
        if n > MAX_LINES {
            return fail(Code::Limit, n, "Source selection excerpt exceeds 128 lines");
        }
        if line.chars().any(|c| c.is_control() && c != '\t') {
            return fail(
                Code::Unsupported,
                n,
                "Use ordinary text with LF or CRLF lines, not control characters",
            );
        }
        if line.starts_with('#') {
            continue;
        }
        if line.trim().is_empty() {
            finish(&mut fields, &mut out, n)?;
            last = None;
            continue;
        }
        if line.starts_with([' ', '\t']) {
            let Some(key) = last else {
                return fail(
                    Code::Syntax,
                    n,
                    "A continued value must follow a named field",
                );
            };
            let span = line.trim();
            let value = fields.get_mut(&key).expect("last field exists");
            if value.len() + 1 + span.len() > 512 {
                return fail(Code::Limit, n, "A source field exceeds 512 bytes");
            }
            value.push(' ');
            value.push_str(span);
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            return fail(
                Code::Unsupported,
                n,
                "Use deb822 named fields; legacy one-line sources are not part of this activity",
            );
        };
        if name.len() > 32 {
            return fail(Code::Unsupported, n, "Unknown source selection field");
        }
        let key = match name.to_ascii_lowercase().as_str() {
            "types" => Field::Types,
            "uris" => Field::Uris,
            "suites" => Field::Suites,
            "components" => Field::Components,
            "enabled" => Field::Enabled,
            _ => {
                return fail(
                    Code::Unsupported,
                    n,
                    "Only Types, URIs, Suites, Components and Enabled belong to this supplied selection excerpt; trust and other policies are outside it",
                );
            }
        };
        if fields.contains_key(&key) {
            return fail(
                Code::Unsupported,
                n,
                "Repeated field names are outside this excerpt; combine values in one field",
            );
        }
        let value = value.trim();
        if value.len() > 512 {
            return fail(Code::Limit, n, "A source field exceeds 512 bytes");
        }
        fields.insert(key, value.into());
        last = Some(key);
    }
    finish(&mut fields, &mut out, count.max(1))?;
    Ok(out)
}
/// Ordered Cartesian targets, deduplicated without changing source preference.
/// Empty or disabled configuration is valid and yields no active indexes.
pub fn select(stanzas: &[Stanza]) -> Result<Selection> {
    if stanzas.len() > MAX_STANZAS {
        return fail(Code::Limit, 0, "At most four source stanzas are supported");
    }
    let mut keys = vec![];
    let mut seen = BTreeSet::new();
    let mut repeated_targets = 0;
    for s in stanzas {
        s.validate()?;
        if !s.enabled {
            continue;
        }
        for u in &s.uris {
            let u = uri(u)?;
            for suite in &s.suites {
                for component in &s.components {
                    let key = Key {
                        uri: u.clone(),
                        suite: suite.clone(),
                        component: component.clone(),
                    };
                    if seen.insert(key.clone()) {
                        if keys.len() >= MAX_KEYS {
                            return fail(Code::Limit, 0, "Source target count exceeds 24");
                        }
                        keys.push(key);
                    } else {
                        repeated_targets += 1;
                    }
                }
            }
        }
    }
    Ok(Selection {
        keys,
        repeated_targets,
    })
}
