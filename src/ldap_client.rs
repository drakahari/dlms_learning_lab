//! In-process virtual ldap.conf and a deliberately bounded ldapsearch interface.
use crate::ldap_query as q;
use q::{Dn, Error, Filter, Scope};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_CONFIG: usize = 8192;
pub const MAX_COMMAND: usize = 4096;
pub const MAX_OUTPUT: usize = 32 * 1024;
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub uri: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub base: String,
}
/// Normalize only the supported URI shape. It selects a virtual supplied directory.
pub fn uri_key(raw: &str) -> Result<String> {
    if raw.len() > 256 {
        return Err(Error::Limit("Directory URI exceeds 256 bytes.".into()));
    }
    let Some(host) = raw.strip_prefix("ldap://") else {
        return Err(Error::Unsupported("Use a supplied ldap:// directory identifier. TLS, IPC and other schemes are not simulated.".into()));
    };
    let host = host.strip_suffix('/').unwrap_or(host);
    if host.is_empty()
        || host.contains(['@', '/', '?', '#', '[', ']'])
        || !host
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".-:".contains(&c))
    {
        return Err(Error::Unsupported("Use ldap://host with an optional numeric port; credentials, paths and URL search components are not supported.".into()));
    }
    let (name, port) = match host.split_once(':') {
        Some((n, p)) => {
            let port = p
                .parse::<u16>()
                .ok()
                .filter(|n| *n != 0)
                .ok_or_else(|| Error::Syntax("Use a port from 1 through 65535.".into()))?;
            (n, port)
        }
        None => (host, 389),
    };
    if name.len() > 253
        || name.split('.').any(|p| {
            p.is_empty()
                || p.len() > 63
                || !p.as_bytes()[0].is_ascii_alphanumeric()
                || !p.as_bytes()[p.len() - 1].is_ascii_alphanumeric()
        })
    {
        return Err(Error::Syntax(
            "Use a supported ASCII host name without empty labels.".into(),
        ));
    }
    Ok(if port == 389 {
        format!("ldap://{}", name.to_ascii_lowercase())
    } else {
        format!("ldap://{}:{port}", name.to_ascii_lowercase())
    })
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err(Error::Unsupported(
                "Client settings require version 1.".into(),
            ));
        }
        uri_key(&self.uri)?;
        Dn::parse(&self.base)?;
        Ok(())
    }
    pub fn parse(raw: &str) -> Result<Self> {
        if raw.len() > MAX_CONFIG {
            return Err(Error::Limit("Client configuration exceeds 8 KiB.".into()));
        }
        if raw
            .chars()
            .any(|c| !c.is_ascii() || c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        {
            return Err(Error::Unsupported(
                "Configuration uses ASCII text, line endings and tabs.".into(),
            ));
        }
        let mut uri = None;
        let mut base = None;
        for (i, line) in raw.lines().enumerate() {
            if i >= 64 || line.len() > 1200 {
                return Err(Error::Limit(
                    "Use at most 64 configuration lines of 1200 bytes.".into(),
                ));
            }
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some(split) = line.find(char::is_whitespace) else {
                return Err(Error::Syntax(format!(
                    "Line {} needs an option and a value.",
                    i + 1
                )));
            };
            let (option, value) = line.split_at(split);
            let value = value.trim();
            let target = if option.eq_ignore_ascii_case("URI") {
                &mut uri
            } else if option.eq_ignore_ascii_case("BASE") {
                &mut base
            } else {
                return Err(Error::Unsupported(format!(
                    "Line {}: only URI and BASE are supported; other options are not silently ignored.",
                    i + 1
                )));
            };
            if target.is_some() {
                return Err(Error::Unsupported(format!(
                    "Line {} repeats an option; use one explicit URI and BASE in this subset.",
                    i + 1
                )));
            }
            if value.is_empty() {
                return Err(Error::Syntax(format!("Line {} needs a value.", i + 1)));
            }
            *target = Some(value.to_string());
        }
        let s = Self {
            version: 1,
            uri: uri
                .ok_or_else(|| Error::Syntax("Set URI to a supplied virtual directory.".into()))?,
            base: base.ok_or_else(|| {
                Error::Syntax("Set BASE to a directory entry's distinguished name.".into())
            })?,
        };
        s.validate()?;
        Ok(s)
    }
}

// Tokenization of a disclosed literal command line, not a general host shell.
// Single quotes retain LDAP backslashes; double quotes implement literal shell escapes.
// Shared literal lexer: visibility only; existing search semantics stay unchanged.
pub(crate) fn words(raw: &str) -> Result<Vec<String>> {
    if raw.len() > MAX_COMMAND {
        return Err(Error::Limit(
            "A search command is limited to 4096 bytes.".into(),
        ));
    }
    if raw.bytes().any(|b| !(32..=126).contains(&b) && b != b'\t') {
        return Err(Error::Unsupported(
            "Enter one ASCII command line; multiline commands and expansions are not supported."
                .into(),
        ));
    }
    let b = raw.as_bytes();
    let mut at = 0;
    let mut result = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut started = false;
    while at < b.len() {
        let c = b[at];
        match quote {
            Some(b'\'') => {
                if c == b'\'' {
                    quote = None;
                } else {
                    word.push(c as char);
                }
                started = true;
                at += 1;
            }
            Some(b'"') => {
                if c == b'"' {
                    quote = None;
                    at += 1;
                } else if c == b'\\' {
                    let Some(&next) = b.get(at + 1) else {
                        return Err(Error::Syntax("Finish the quoted escape.".into()));
                    };
                    if b"$`\"\\".contains(&next) {
                        word.push(next as char);
                        at += 2;
                    } else {
                        word.push('\\');
                        at += 1;
                    }
                } else if matches!(c, b'$' | b'`') {
                    return Err(Error::Unsupported(
                        "Shell expansion is outside this literal search-command subset.".into(),
                    ));
                } else {
                    word.push(c as char);
                    at += 1;
                }
                started = true;
            }
            None => {
                if matches!(c, b' ' | b'\t') {
                    if started {
                        result.push(std::mem::take(&mut word));
                        started = false;
                    }
                    at += 1;
                } else if matches!(c, b'\'' | b'"') {
                    quote = Some(c);
                    started = true;
                    at += 1;
                } else if c == b'\\' {
                    let Some(&next) = b.get(at + 1) else {
                        return Err(Error::Syntax("Finish the command escape.".into()));
                    };
                    word.push(next as char);
                    started = true;
                    at += 2;
                } else if b";|&<>()$`*?[]{}#~".contains(&c) {
                    return Err(Error::Unsupported("Use one literal ldapsearch command. Quote filters and selectors; pipelines, globbing, expansion and shell operators are not supported here.".into()));
                } else {
                    word.push(c as char);
                    started = true;
                    at += 1;
                }
            }
            _ => unreachable!(),
        }
        if result.len() > 64 {
            return Err(Error::Limit("Use at most 64 command arguments.".into()));
        }
    }
    if quote.is_some() {
        return Err(Error::Syntax("Close the quoted command argument.".into()));
    }
    if started {
        result.push(word);
    }
    if result.len() > 64 {
        return Err(Error::Limit("Use at most 64 command arguments.".into()));
    }
    Ok(result)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub uri: String,
    pub base: Dn,
    pub scope: Scope,
    pub filter: Filter,
    /// Empty means all supplied user attributes; 1.1 alone means none.
    pub attributes: Vec<String>,
}
impl Query {
    pub fn parse(settings: &Settings, raw: &str) -> Result<Self> {
        settings.validate()?;
        let args = words(raw)?;
        if args.first().map(String::as_str) != Some("ldapsearch") {
            return Err(Error::Unsupported("Supported command: ldapsearch -x -LLL [-H URI] [-b DN] [-s base|one|sub] 'FILTER' [attributes]. No host command is run.".into()));
        }
        let mut i = 1;
        let mut uri = settings.uri.clone();
        let mut base = settings.base.clone();
        let mut scope = Scope::Sub;
        let mut seen = BTreeSet::new();
        while let Some(arg) = args.get(i) {
            if !arg.starts_with('-') {
                break;
            }
            if !seen.insert(arg.as_str()) {
                return Err(Error::Unsupported(
                    "State each search option once in this subset.".into(),
                ));
            }
            match arg.as_str() {
                "-x" | "-LLL" => i += 1,
                "-H" | "-b" | "-s" => {
                    let v = args
                        .get(i + 1)
                        .ok_or_else(|| Error::Syntax(format!("{arg} needs a value.")))?;
                    match arg.as_str() {
                        "-H" => uri = v.clone(),
                        "-b" => base = v.clone(),
                        _ => scope = Scope::parse(v)?,
                    }
                    i += 2;
                }
                _ => {
                    return Err(Error::Unsupported(format!(
                        "Option {arg} is not supported. Only -x, -LLL, -H, -b and -s are implemented; no passwords, files or real connections."
                    )));
                }
            }
        }
        if !seen.contains("-x") || !seen.contains("-LLL") {
            return Err(Error::Unsupported("Include -x -LLL for the simulated anonymous simple search and plain LDIF output. SASL/bind authentication and other output modes are not implemented.".into()));
        }
        let filter = Filter::parse(args.get(i).ok_or_else(|| {
            Error::Syntax("Write an explicit quoted filter, such as '(objectClass=*)'.".into())
        })?)?;
        i += 1;
        let mut attributes = Vec::new();
        let mut selected = BTreeSet::new();
        for attr in &args[i..] {
            let a = attr.to_ascii_lowercase();
            if a != "*"
                && a != "1.1"
                && (!a.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
                    || a.len() > 32
                    || !a.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-'))
            {
                return Err(Error::Unsupported("Select named attributes, quoted '*' or 1.1. Operational attributes, options and OID aliases are not implemented.".into()));
            }
            if selected.insert(a.clone()) {
                attributes.push(a);
            }
        }
        Ok(Self {
            uri: uri_key(&uri)?,
            base: Dn::parse(&base)?,
            scope,
            filter,
            attributes,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultEntry {
    pub dn: String,
    pub attributes: Vec<q::Attribute>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Entries(Vec<ResultEntry>),
    NoSuchObject,
    Unavailable { uri: String },
}
#[derive(Clone, Debug)]
pub struct Environment {
    directories: Vec<(String, q::Directory)>,
}
impl Environment {
    pub fn new(sources: &[(String, q::Source)]) -> Result<Self> {
        if sources.is_empty() || sources.len() > 4 {
            return Err(Error::Limit(
                "Supply one to four virtual directories.".into(),
            ));
        }
        let mut directories = Vec::new();
        let mut seen = BTreeSet::new();
        for (uri, source) in sources {
            let uri = uri_key(uri)?;
            if !seen.insert(uri.clone()) {
                return Err(Error::Fixture(
                    "Two virtual directories use the same URI.".into(),
                ));
            }
            directories.push((uri, q::Directory::new(source)?));
        }
        Ok(Self { directories })
    }
    pub fn search(&self, query: &Query) -> Result<Outcome> {
        // Query fields are inspectable, so revalidate URI and selector shape at this boundary.
        let uri = uri_key(&query.uri)?;
        if query.attributes.len() > 64
            || query.attributes.iter().any(|a| {
                a.len() > 32
                    || a != "*"
                        && a != "1.1"
                        && (!a.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
                            || !a.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-'))
            })
        {
            return Err(Error::Unsupported(
                "Use bounded supported attribute selectors.".into(),
            ));
        }
        let Some((_, d)) = self.directories.iter().find(|(u, _)| *u == uri) else {
            return Ok(Outcome::Unavailable { uri });
        };
        let q::SearchResult::Entries(entries) = d.search(&query.base, query.scope, &query.filter)
        else {
            return Ok(Outcome::NoSuchObject);
        };
        let all = query.attributes.is_empty() || query.attributes.iter().any(|a| a == "*");
        let rows = entries
            .iter()
            .map(|e| ResultEntry {
                dn: e.dn().text().into(),
                attributes: e
                    .attributes()
                    .iter()
                    .filter(|a| {
                        all || query
                            .attributes
                            .iter()
                            .any(|s| s.eq_ignore_ascii_case(&a.name))
                    })
                    .cloned()
                    .collect(),
            })
            .collect();
        let outcome = Outcome::Entries(rows);
        outcome.output()?;
        Ok(outcome)
    }
    pub fn run(&self, settings: &Settings, raw: &str) -> Result<Outcome> {
        self.search(&Query::parse(settings, raw)?)
    }
}
fn base64(raw: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::new();
    for b in raw.chunks(3) {
        let x = (u32::from(b[0]) << 16)
            | (u32::from(*b.get(1).unwrap_or(&0)) << 8)
            | u32::from(*b.get(2).unwrap_or(&0));
        s.push(A[((x >> 18) & 63) as usize] as char);
        s.push(A[((x >> 12) & 63) as usize] as char);
        s.push(if b.len() > 1 {
            A[((x >> 6) & 63) as usize] as char
        } else {
            '='
        });
        s.push(if b.len() > 2 {
            A[(x & 63) as usize] as char
        } else {
            '='
        });
    }
    s
}
fn ldif_line(name: &str, value: &str) -> String {
    if value.starts_with([' ', ':', '<']) || value.ends_with(' ') {
        format!("{name}:: {}\n", base64(value.as_bytes()))
    } else {
        format!("{name}: {value}\n")
    }
}
impl Outcome {
    pub fn succeeded(&self) -> bool {
        matches!(self, Self::Entries(_))
    }
    pub fn output(&self) -> Result<String> {
        let mut out = String::new();
        match self {
            Self::NoSuchObject => out.push_str(
                "No such object (32): the search base is absent from the supplied directory.\n",
            ),
            Self::Unavailable { uri } => {
                if uri.len() > 256 {
                    return Err(Error::Limit("URI exceeds output bound.".into()));
                }
                out.push_str(&format!(
                    "Simulated directory unavailable: {uri}\nNo host connection was attempted.\n"
                ));
            }
            Self::Entries(entries) => {
                if entries.len() > q::MAX_ENTRIES {
                    return Err(Error::Limit("Result contains too many entries.".into()));
                }
                for e in entries {
                    if e.dn.len() > 1024
                        || e.attributes.len() > 16
                        || !e.dn.bytes().all(|c| (32..=126).contains(&c))
                    {
                        return Err(Error::Limit("Result entry exceeds bounds.".into()));
                    }
                    out.push_str(&ldif_line("dn", &e.dn));
                    for a in &e.attributes {
                        if a.name.len() > 32
                            || a.values.len() > 16
                            || !a
                                .name
                                .bytes()
                                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
                        {
                            return Err(Error::Limit("Result attribute exceeds bounds.".into()));
                        }
                        for v in &a.values {
                            if v.len() > 256 || !v.bytes().all(|c| (32..=126).contains(&c)) {
                                return Err(Error::Limit("Result value exceeds bounds.".into()));
                            }
                            out.push_str(&ldif_line(&a.name, v));
                            if out.len() > MAX_OUTPUT {
                                return Err(Error::Limit("Search output exceeds 32 KiB; request fewer attributes or narrow the search.".into()));
                            }
                        }
                    }
                    out.push('\n');
                    if out.len() > MAX_OUTPUT {
                        return Err(Error::Limit(
                            "Search output exceeds 32 KiB; narrow the search.".into(),
                        ));
                    }
                }
            }
        }
        Ok(out)
    }
}
