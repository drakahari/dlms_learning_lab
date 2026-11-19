//! Bounded, ordered static LDAP ACL decisions. No server, I/O or authentication.
//! Callers supply already established subject facts; this is not a credential API.
use crate::ldap_query::{Dn, Scope};
use std::fmt;

pub const MAX_BYTES: usize = 32 * 1024;
pub const MAX_LINES: usize = 256;
pub const MAX_RULES: usize = 16;
pub const MAX_CLAUSES: usize = 16;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Syntax(usize, &'static str),
    Unsupported(usize, &'static str),
    Limit(&'static str),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(line, s) => write!(f, "Syntax at line {line}: {s}"),
            Self::Unsupported(line, s) => write!(f, "Unsupported at line {line}: {s}"),
            Self::Limit(s) => write!(f, "Limit: {s}"),
        }
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    None,
    Disclose,
    Auth,
    Compare,
    Search,
    Read,
    Write,
    Manage,
}
impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "no",
            Self::Disclose => "disclose",
            Self::Auth => "authentication",
            Self::Compare => "compare",
            Self::Search => "search",
            Self::Read => "read",
            Self::Write => "write",
            Self::Manage => "manage",
        }
    }
    fn parse(s: &str, line: usize) -> Result<Self> {
        match s {
            "none" => Ok(Self::None),
            "disclose" => Ok(Self::Disclose),
            "auth" => Ok(Self::Auth),
            "compare" => Ok(Self::Compare),
            "search" => Ok(Self::Search),
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            "manage" => Ok(Self::Manage),
            _ => Err(Error::Unsupported(
                line,
                "Use a supported lowercase access level; privilege masks and selfwrite are outside this subset.",
            )),
        }
    }
}
/// None is a granted level, never a useful requested privilege.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    Disclose,
    Auth,
    Compare,
    Search,
    Read,
    Write,
    Manage,
}
impl Request {
    fn level(self) -> Level {
        match self {
            Self::Disclose => Level::Disclose,
            Self::Auth => Level::Auth,
            Self::Compare => Level::Compare,
            Self::Search => Level::Search,
            Self::Read => Level::Read,
            Self::Write => Level::Write,
            Self::Manage => Level::Manage,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Subject {
    Anonymous,
    Authenticated(Dn),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Selection {
    Base,
    One,
    Subtree,
    Children,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct NameSelection {
    scope: Selection,
    dn: Dn,
}
impl NameSelection {
    fn matches(&self, dn: &Dn) -> bool {
        match self.scope {
            Selection::Base => dn.equivalent(&self.dn),
            Selection::One => dn.within(&self.dn, Scope::One),
            Selection::Subtree => dn.within(&self.dn, Scope::Sub),
            Selection::Children => dn.within(&self.dn, Scope::Sub) && !dn.equivalent(&self.dn),
        }
    }
    fn parse(s: &str, line: usize) -> Result<Self> {
        let (kind, text) = s.split_once('=').ok_or(Error::Syntax(
            line,
            "A DN selector needs = and a complete name.",
        ))?;
        let scope = match kind {
            "dn.exact" | "dn.base" => Selection::Base,
            "dn.one" => Selection::One,
            "dn.subtree" => Selection::Subtree,
            "dn.children" => Selection::Children,
            _ => {
                return Err(Error::Unsupported(
                    line,
                    "Use dn.exact/base/one/subtree/children; regex and unqualified DN selectors are outside this subset.",
                ));
            }
        };
        let dn = Dn::parse(text).map_err(|_| {
            Error::Unsupported(
                line,
                "This DN is outside the declared printable-ASCII naming subset.",
            )
        })?;
        Ok(Self { scope, dn })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Who {
    All,
    Anonymous,
    Users,
    SelfEntry,
    Name(NameSelection),
}
impl Who {
    fn parse(s: &str, line: usize) -> Result<Self> {
        match s {
            "*" => Ok(Self::All),
            "anonymous" => Ok(Self::Anonymous),
            "users" => Ok(Self::Users),
            "self" => Ok(Self::SelfEntry),
            s if s.starts_with("dn.") => Ok(Self::Name(NameSelection::parse(s, line)?)),
            _ => Err(Error::Unsupported(
                line,
                "Use *, anonymous, users, self or a supported DN selector; group, regex, SSF, sets and other selectors are not implemented.",
            )),
        }
    }
    fn matches(&self, subject: &Subject, target: &Dn) -> bool {
        match (self, subject) {
            (Self::All, _) | (Self::Anonymous, Subject::Anonymous) => true,
            (Self::Users, Subject::Authenticated(_)) => true,
            (Self::SelfEntry, Subject::Authenticated(dn)) => dn.equivalent(target),
            (Self::Name(selection), Subject::Authenticated(dn)) => selection.matches(dn),
            _ => false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Rule {
    line: usize,
    name: Option<NameSelection>,
    attributes: Vec<String>,
    clauses: Vec<(Who, Level)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    rules: Vec<Rule>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    RootDn,
    EmptyDefaultRead,
    Matched,
    NoSubjectMatch,
    NoTargetMatch,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub allowed: bool,
    pub granted: Level,
    /// One-based source rule and by-clause numbers, absent for implicit defaults.
    pub rule: Option<usize>,
    pub clause: Option<usize>,
    pub source_line: Option<usize>,
    pub reason: Reason,
}
impl Decision {
    pub fn explanation(&self) -> String {
        match self.reason {
            Reason::RootDn => "The established identity is the configured rootDN and bypasses ordinary access rules.".into(),
            Reason::EmptyDefaultRead => "No access rules are configured, so the default allows reading. Changing a field still needs write access.".into(),
            Reason::Matched => format!("Rule {}, caller clause {} grants {} access. Later rules do not add permissions after this match.", self.rule.unwrap_or(0), self.clause.unwrap_or(0), self.granted.label()),
            Reason::NoSubjectMatch => format!("Rule {} matches this entry and field, but none of its caller clauses match. Its implied default is no access.", self.rule.unwrap_or(0)),
            Reason::NoTargetMatch => "No rule matches this entry and field. A nonempty policy denies unmatched requests.".into(),
        }
    }
}
fn attribute(s: &str) -> bool {
    s.len() <= 32
        && matches!(
            s.to_ascii_lowercase().as_str(),
            "cn" | "sn"
                | "ou"
                | "uid"
                | "dc"
                | "mail"
                | "description"
                | "objectclass"
                | "userpassword"
                | "entry"
                | "children"
        )
}
// Preserve literal DN values. Configuration escaping/embedded quotes are refused
// rather than guessed; quotes may surround only the value of a DN selector.
fn words(s: &str, line: usize) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let mut token = String::new();
    let mut quoted = false;
    let mut closed = false;
    for c in s.chars() {
        if c == '\\' {
            return Err(Error::Unsupported(
                line,
                "Configuration escapes are outside this subset; use supplied unescaped names.",
            ));
        }
        if c == '"' {
            if quoted {
                quoted = false;
                closed = true;
            } else if !closed && token.starts_with("dn.") && token.ends_with('=') {
                quoted = true;
            } else {
                return Err(Error::Unsupported(
                    line,
                    "Only a complete DN selector value may be double-quoted.",
                ));
            }
        } else if c.is_ascii_whitespace() && !quoted {
            if !token.is_empty() {
                out.push(std::mem::take(&mut token));
                closed = false;
            }
        } else {
            if closed {
                return Err(Error::Syntax(
                    line,
                    "Separate the quoted DN from the next option with whitespace.",
                ));
            }
            if c == '#' && !quoted {
                return Err(Error::Unsupported(
                    line,
                    "Use full-line comments; inline comment handling is outside this subset.",
                ));
            }
            token.push(c);
        }
        if out.len() > 1024 || token.len() > 1200 {
            return Err(Error::Limit(
                "An ACL has at most 1024 tokens, each at most 1200 bytes.",
            ));
        }
    }
    if quoted {
        return Err(Error::Syntax(line, "Close the quoted DN value."));
    }
    if !token.is_empty() {
        out.push(token);
    }
    if out.len() > 1024 {
        return Err(Error::Limit("An ACL has at most 1024 tokens."));
    }
    Ok(out)
}
impl Rule {
    fn parse(s: &str, line: usize) -> Result<Self> {
        let w = words(s, line)?;
        if w.first().map(String::as_str) != Some("access")
            || w.get(1).map(String::as_str) != Some("to")
        {
            return Err(Error::Unsupported(
                line,
                "Start a lowercase access to rule; full slapd configuration and olcAccess LDIF are separate unsupported formats.",
            ));
        }
        let mut at = 2;
        let mut name = None;
        let mut attrs = Vec::new();
        let mut star = false;
        while let Some(t) = w.get(at).filter(|t| t.as_str() != "by") {
            if t == "*" && name.is_none() && attrs.is_empty() && !star {
                star = true;
            } else if t.starts_with("dn.") && name.is_none() && attrs.is_empty() && !star {
                name = Some(NameSelection::parse(t, line)?);
            } else if let Some(list) = t
                .strip_prefix("attrs=")
                .filter(|_| attrs.is_empty() && !star)
            {
                for a in list.split(',') {
                    let a = a.to_ascii_lowercase();
                    if !attribute(&a) || attrs.contains(&a) {
                        return Err(Error::Unsupported(
                            line,
                            "Use distinct supported attribute names; options, OIDs and value/filter selectors are outside this subset.",
                        ));
                    }
                    attrs.push(a);
                    if attrs.len() > 11 {
                        return Err(Error::Limit("At most 11 declared attributes per rule."));
                    }
                }
            } else {
                return Err(Error::Unsupported(
                    line,
                    "Unsupported or repeated target selector; use one DN and/or one attrs list, or * alone.",
                ));
            }
            at += 1;
        }
        if !star && name.is_none() && attrs.is_empty() {
            return Err(Error::Syntax(
                line,
                "Name a target, attributes, or * before by.",
            ));
        }
        let mut clauses = Vec::new();
        while at < w.len() {
            if w[at] != "by" {
                return Err(Error::Unsupported(
                    line,
                    "Only stop behavior is supported; continue/break and privilege accumulation are not implemented.",
                ));
            }
            let who = w.get(at + 1).ok_or(Error::Syntax(
                line,
                "Each by needs a subject and access level.",
            ))?;
            let level = w
                .get(at + 2)
                .ok_or(Error::Syntax(line, "Each by needs an access level."))?;
            clauses.push((Who::parse(who, line)?, Level::parse(level, line)?));
            if clauses.len() > MAX_CLAUSES {
                return Err(Error::Limit("An ACL rule has at most 16 caller clauses."));
            }
            at += 3;
            if w.get(at).is_some_and(|w| w == "stop") {
                at += 1;
            }
        }
        if clauses.is_empty() {
            return Err(Error::Syntax(
                line,
                "Each access rule needs at least one by clause.",
            ));
        }
        Ok(Self {
            line,
            name,
            attributes: attrs,
            clauses,
        })
    }
}
impl Policy {
    /// One backend's already combined list, not a whole slapd.conf or cn=config.
    /// Headers start in column1; continuation lines must be indented by clauses.
    pub fn parse(text: &str) -> Result<Self> {
        if text.len() > MAX_BYTES {
            return Err(Error::Limit("ACL text is limited to 32 KiB."));
        }
        if text
            .as_bytes()
            .iter()
            .enumerate()
            .any(|(at, b)| *b == b'\r' && text.as_bytes().get(at + 1) != Some(&b'\n'))
        {
            return Err(Error::Unsupported(
                1,
                "Use LF or CRLF; bare carriage returns are outside this subset.",
            ));
        }
        if text
            .bytes()
            .any(|b| !matches!(b, 32..=126 | b'\n' | b'\r' | b'\t'))
        {
            return Err(Error::Unsupported(
                1,
                "ACL text uses printable ASCII, LF/CRLF and tabs only.",
            ));
        }
        let mut rules = Vec::new();
        let mut current = String::new();
        let mut start = 0;
        for (n, raw) in text.lines().enumerate() {
            let line = n + 1;
            if line > MAX_LINES || raw.len() > 4096 {
                return Err(Error::Limit("At most 256 lines, each at most 4096 bytes."));
            }
            let trimmed = raw.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.contains('\r') {
                return Err(Error::Unsupported(
                    line,
                    "Bare carriage returns are not a line separator in this subset.",
                ));
            }
            if raw.starts_with(' ') || raw.starts_with('\t') {
                if current.is_empty() || !trimmed.starts_with("by ") && !trimmed.starts_with("by\t")
                {
                    return Err(Error::Unsupported(
                        line,
                        "Continuation lines must start with an indented by clause.",
                    ));
                }
                current.push(' ');
                current.push_str(trimmed);
            } else {
                if !current.is_empty() {
                    rules.push(Rule::parse(&current, start)?);
                }
                if rules.len() >= MAX_RULES {
                    return Err(Error::Limit("At most 16 access rules are supported."));
                }
                current = trimmed.into();
                start = line;
            }
        }
        if !current.is_empty() {
            rules.push(Rule::parse(&current, start)?);
        }
        Ok(Self { rules })
    }
    /// Evaluate a single attribute/pseudo-attribute privilege. This is not an LDAP
    /// operation: callers must additionally check operation-level entry/children
    /// prerequisites, schema and authentication before executing any request.
    pub fn evaluate(
        &self,
        subject: &Subject,
        target: &Dn,
        attr: &str,
        request: Request,
        root_dn: Option<&Dn>,
    ) -> Result<Decision> {
        if !attribute(attr) {
            return Err(Error::Unsupported(
                0,
                "Choose a declared attribute, entry or children; unknown schema attributes are outside this subset.",
            ));
        }
        let result = |granted, rule, clause, source_line, reason| Decision {
            allowed: granted >= request.level(),
            granted,
            rule,
            clause,
            source_line,
            reason,
        };
        if matches!(subject,Subject::Authenticated(dn)if root_dn.is_some_and(|root|dn.equivalent(root)))
        {
            return Ok(result(Level::Manage, None, None, None, Reason::RootDn));
        }
        if self.rules.is_empty() {
            return Ok(result(
                Level::Read,
                None,
                None,
                None,
                Reason::EmptyDefaultRead,
            ));
        }
        for (index, rule) in self.rules.iter().enumerate() {
            if rule.name.as_ref().is_some_and(|name| !name.matches(target))
                || !rule.attributes.is_empty()
                    && !rule.attributes.iter().any(|a| a.eq_ignore_ascii_case(attr))
            {
                continue;
            }
            for (clause, (who, level)) in rule.clauses.iter().enumerate() {
                if who.matches(subject, target) {
                    return Ok(result(
                        *level,
                        Some(index + 1),
                        Some(clause + 1),
                        Some(rule.line),
                        Reason::Matched,
                    ));
                }
            }
            return Ok(result(
                Level::None,
                Some(index + 1),
                None,
                Some(rule.line),
                Reason::NoSubjectMatch,
            ));
        }
        Ok(result(Level::None, None, None, None, Reason::NoTargetMatch))
    }
}
