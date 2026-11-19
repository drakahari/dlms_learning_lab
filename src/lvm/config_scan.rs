//! Pure bounded device-filter configuration prerequisite. No normal source/UI
//! admission, host configuration, devices, utilities or I/O. No upstream C copied.
use super::{DeviceRole, State};
use crate::ere;
use std::collections::BTreeSet;
pub const MAX_CONFIG: usize = 4096;
pub const MAX_RULES: usize = 8;
pub const MAX_PATTERN: usize = 128;
pub const MAX_MATCH_WORK: usize = 65_536;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Syntax,
    Unsupported,
    Limit,
    InvalidContext,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: Kind,
    pub byte: Option<usize>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)?;
        if let (Some(line), Some(column)) = (self.line, self.column) {
            write!(f, " (line {line}, column {column})")?;
        } else if let Some(at) = self.byte {
            write!(f, " (position {})", at + 1)?;
        }
        Ok(())
    }
}
impl std::error::Error for Diagnostic {}
type Result<T> = std::result::Result<T, Diagnostic>;
fn diagnostic(kind: Kind, byte: Option<usize>, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        kind,
        byte,
        line: None,
        column: None,
        message: message.into(),
    }
}
fn expression_error(error: ere::Error, at: Option<usize>) -> Diagnostic {
    let kind = match error.kind {
        ere::ErrorKind::Syntax => Kind::Syntax,
        ere::ErrorKind::Unsupported => Kind::Unsupported,
        ere::ErrorKind::Limit => Kind::Limit,
    };
    let byte = match (at, error.byte) {
        (Some(base), Some(offset)) => Some(base + offset),
        (Some(base), None) => Some(base),
        _ => None,
    };
    let message = if kind == Kind::Limit {
        "This filter expression exceeds the unit's work or size limit. Use a simpler pattern."
            .into()
    } else {
        format!("Filter expression: {}", error.message)
    };
    diagnostic(kind, byte, message)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Accept,
    Reject,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub action: Action,
    pub pattern: String,
}
/// Constructed only by the closed parser. Save raw text, not a trusted AST, when
/// a later source/runtime profile admits this capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    use_devices_file: bool,
    rules: Vec<Rule>,
}
impl Policy {
    pub fn uses_devices_file(&self) -> bool {
        self.use_devices_file
    }
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }
}
struct Parser<'a> {
    text: &'a str,
    at: usize,
}
impl<'a> Parser<'a> {
    fn fail(&self, kind: Kind, message: &str) -> Diagnostic {
        diagnostic(kind, Some(self.at), message)
    }
    fn space(&mut self) {
        loop {
            while self
                .text
                .as_bytes()
                .get(self.at)
                .is_some_and(u8::is_ascii_whitespace)
            {
                self.at += 1;
            }
            if self.text.as_bytes().get(self.at) != Some(&b'#') {
                break;
            }
            while self
                .text
                .as_bytes()
                .get(self.at)
                .is_some_and(|b| *b != b'\n')
            {
                self.at += 1;
            }
        }
    }
    fn expect(&mut self, byte: u8) -> Result<()> {
        self.space();
        if self.text.as_bytes().get(self.at) != Some(&byte) {
            return Err(self.fail(Kind::Syntax, &format!("Expected '{}'", char::from(byte))));
        }
        self.at += 1;
        Ok(())
    }
    fn identifier(&mut self) -> Result<&'a str> {
        self.space();
        let start = self.at;
        if !self
            .text
            .as_bytes()
            .get(self.at)
            .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
        {
            return Err(self.fail(Kind::Syntax, "Expected a setting or section name"));
        }
        while self
            .text
            .as_bytes()
            .get(self.at)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
        {
            self.at += 1;
        }
        Ok(&self.text[start..self.at])
    }
    fn flag(&mut self) -> Result<bool> {
        self.space();
        let start = self.at;
        while self
            .text
            .as_bytes()
            .get(self.at)
            .is_some_and(u8::is_ascii_digit)
        {
            self.at += 1;
        }
        if self
            .text
            .as_bytes()
            .get(self.at)
            .is_some_and(|b| !b.is_ascii_whitespace() && !b"#}".contains(b))
        {
            return Err(self.fail(
                Kind::Syntax,
                "Separate the integer value from the next setting",
            ));
        }
        match &self.text[start..self.at] {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(diagnostic(
                Kind::Unsupported,
                Some(start),
                "This unit supports explicit use_devicesfile = 0 or 1 only",
            )),
        }
    }
    fn quoted(&mut self) -> Result<(&'a str, usize)> {
        self.expect(b'"')?;
        let start = self.at;
        while let Some(&byte) = self.text.as_bytes().get(self.at) {
            if self.at - start > MAX_PATTERN + 3 {
                return Err(self.fail(
                    Kind::Limit,
                    "Filter entry exceeds the unit's 128-byte expression allowance",
                ));
            }
            match byte {
                b'"' => {
                    let s = &self.text[start..self.at];
                    self.at += 1;
                    return Ok((s, start));
                }
                b'\\' => {
                    return Err(self.fail(
                        Kind::Unsupported,
                        "Quoted backslash escapes are outside this unit; they are not evaluated",
                    ));
                }
                b if b.is_ascii_control() => {
                    return Err(self.fail(
                        Kind::Unsupported,
                        "Filter strings use printable ASCII without line breaks",
                    ));
                }
                _ => self.at += 1,
            }
        }
        Err(self.fail(Kind::Syntax, "Close the quoted filter entry"))
    }
    fn rules(&mut self) -> Result<Vec<Rule>> {
        self.expect(b'[')?;
        self.space();
        let mut rules = Vec::new();
        if self.text.as_bytes().get(self.at) == Some(&b']') {
            return Err(self.fail(Kind::Syntax, "Supply at least one quoted filter rule"));
        }
        loop {
            // Refuse the ninth entry before copying or parsing its value.
            if rules.len() >= MAX_RULES {
                return Err(self.fail(Kind::Limit, "At most eight filter rules in this unit"));
            }
            let (entry, start) = self.quoted()?;
            rules.push(rule(entry, start)?);
            self.space();
            match self.text.as_bytes().get(self.at) {
                Some(b']') => {
                    self.at += 1;
                    break;
                }
                Some(b',') => {
                    self.at += 1;
                    self.space();
                    if self.text.as_bytes().get(self.at) == Some(&b']') {
                        return Err(self.fail(
                            Kind::Unsupported,
                            "Trailing array commas are outside this unit",
                        ));
                    }
                }
                _ => {
                    return Err(self.fail(
                        Kind::Syntax,
                        "Separate quoted rules with a comma, then close the array",
                    ));
                }
            }
        }
        Ok(rules)
    }
}
fn rule(entry: &str, start: usize) -> Result<Rule> {
    let b = entry.as_bytes();
    if b.len() < 3 {
        return Err(diagnostic(
            Kind::Syntax,
            Some(start),
            "Use a quoted accept/reject rule with matching delimiters",
        ));
    }
    let action = match b[0] {
        b'a' => Action::Accept,
        b'r' => Action::Reject,
        _ => {
            return Err(diagnostic(
                Kind::Syntax,
                Some(start),
                "A filter rule starts with a for accept or r for reject",
            ));
        }
    };
    let close = match b[1] {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        other => other,
    };
    if b[b.len() - 1] != close {
        return Err(diagnostic(
            Kind::Syntax,
            Some(start + entry.len() - 1),
            "The rule's closing delimiter does not match its opening delimiter",
        ));
    }
    let pattern = &entry[2..entry.len() - 1];
    if pattern.is_empty() {
        return Err(diagnostic(
            Kind::Unsupported,
            Some(start + 2),
            "Empty regular expressions are outside this unit; use an explicit pattern",
        ));
    }
    if pattern.len() > MAX_PATTERN {
        return Err(diagnostic(
            Kind::Limit,
            Some(start + 2),
            "Filter expression exceeds 128 bytes",
        ));
    }
    ere::Regex::compile(pattern).map_err(|e| expression_error(e, Some(start + 2)))?;
    Ok(Rule {
        action,
        pattern: pattern.into(),
    })
}
/// Closed lvm.conf fragment: one devices section, explicit use_devicesfile and
/// filter assignments, comments, whitespace and double-quoted ASCII rules.
/// Missing defaults, other keys/sections, repeats and escapes refuse honestly.
pub fn parse(text: &str) -> Result<Policy> {
    parse_checked(text).map_err(|mut error| {
        if let Some(byte) = error.byte {
            let end = byte.min(text.len());
            let prefix = &text.as_bytes()[..end];
            error.line = Some(prefix.iter().filter(|b| **b == b'\n').count() + 1);
            error.column = Some(
                prefix
                    .iter()
                    .rposition(|b| *b == b'\n')
                    .map_or(end + 1, |last| end - last),
            );
        }
        error
    })
}
fn parse_checked(text: &str) -> Result<Policy> {
    if text.len() > MAX_CONFIG {
        return Err(diagnostic(
            Kind::Limit,
            None,
            "Configuration exceeds 4096 bytes",
        ));
    }
    if let Some(at) = text
        .bytes()
        .position(|b| !b.is_ascii() || b.is_ascii_control() && !b"\r\n\t".contains(&b))
    {
        return Err(diagnostic(
            Kind::Unsupported,
            Some(at),
            "This unit accepts ASCII configuration text with ordinary whitespace only",
        ));
    }
    let mut p = Parser { text, at: 0 };
    if p.identifier()? != "devices" {
        return Err(p.fail(
            Kind::Unsupported,
            "Only the supplied devices section is supported",
        ));
    }
    p.expect(b'{')?;
    let mut use_devices_file = None;
    let mut rules = None;
    loop {
        p.space();
        if p.text.as_bytes().get(p.at) == Some(&b'}') {
            p.at += 1;
            break;
        }
        let key = p.identifier()?;
        p.expect(b'=')?;
        match key {
            "use_devicesfile" if use_devices_file.is_none() => use_devices_file = Some(p.flag()?),
            "filter" if rules.is_none() => rules = Some(p.rules()?),
            "filter" | "use_devicesfile" => {
                return Err(p.fail(
                    Kind::Unsupported,
                    "Repeated settings are outside this unit; keep one explicit value",
                ));
            }
            _ => {
                return Err(p.fail(
                    Kind::Unsupported,
                    "This unit supports devices/use_devicesfile and devices/filter only",
                ));
            }
        }
    }
    p.space();
    if p.at != text.len() {
        return Err(p.fail(
            Kind::Unsupported,
            "Additional sections, statements and override syntax are outside this unit",
        ));
    }
    let (Some(use_devices_file), Some(rules)) = (use_devices_file, rules) else {
        return Err(p.fail(Kind::Unsupported,"Provide explicit use_devicesfile and filter values; implicit distribution defaults are not modeled"));
    };
    Ok(Policy {
        use_devices_file,
        rules,
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevicesFile {
    entries: BTreeSet<String>,
}
impl DevicesFile {
    /// One supplied devname-only devices file, not real UUID/hardware-ID lookup.
    pub fn new(paths: &[String]) -> Result<Self> {
        if paths.len() > super::MAX_DEVICES {
            return Err(diagnostic(
                Kind::Limit,
                None,
                "Devices file exceeds eight supplied names",
            ));
        }
        let mut entries = BTreeSet::new();
        for path in paths {
            if !super::device_name(path) || !entries.insert(path.clone()) {
                return Err(diagnostic(
                    Kind::InvalidContext,
                    None,
                    "Devices file needs distinct bounded virtual /dev/NAME entries",
                ));
            }
        }
        Ok(Self { entries })
    }
    pub fn entries(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(String::as_str)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    FilterSelected,
    FilterBecauseFileMissing,
    DevicesFile,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    FilterRule { index: usize, action: Action },
    NoFilterMatched,
    ListedInDevicesFile,
    NotInDevicesFile,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Metadata {
    PhysicalVolume { group: Option<String> },
    NoPhysicalVolumeLabel,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub path: String,
    pub allowed: bool,
    pub reason: Reason,
    pub metadata: Option<Metadata>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub mode: Mode,
    pub records: Vec<Record>,
    pub unresolved_file_entries: Vec<String>,
    pub matching_work: usize,
}
fn metadata(d: &super::Device) -> Metadata {
    if d.role == DeviceRole::PhysicalVolume {
        Metadata::PhysicalVolume {
            group: (!d.group.is_empty()).then(|| d.group.clone()),
        }
    } else {
        Metadata::NoPhysicalVolumeLabel
    }
}
pub fn scan(policy: &Policy, state: &State, file: Option<&DevicesFile>) -> Result<Report> {
    scan_limited(policy, state, file, MAX_MATCH_WORK)
}
/// One shared matching allowance across every rule and supplied device. Pure
/// failure publishes no partial report and never changes any virtual storage.
pub fn scan_limited(
    policy: &Policy,
    state: &State,
    file: Option<&DevicesFile>,
    allowance: usize,
) -> Result<Report> {
    if allowance > MAX_MATCH_WORK {
        return Err(diagnostic(
            Kind::Limit,
            None,
            "Matching allowance cannot exceed 65536 transitions",
        ));
    }
    if !crate::cron_workspace::within(state, 64 * 1024) {
        return Err(diagnostic(
            Kind::Limit,
            None,
            "Scan inventory exceeds this unit's 64 KiB source bound",
        ));
    }
    state
        .validate()
        .map_err(|e| diagnostic(Kind::InvalidContext, None, e.to_string()))?;
    if policy.use_devices_file
        && let Some(file) = file
    {
        let records = state
            .devices
            .iter()
            .map(|(path, device)| {
                let allowed = file.entries.contains(path);
                Record {
                    path: path.clone(),
                    allowed,
                    reason: if allowed {
                        Reason::ListedInDevicesFile
                    } else {
                        Reason::NotInDevicesFile
                    },
                    metadata: allowed.then(|| metadata(device)),
                }
            })
            .collect();
        return Ok(Report {
            mode: Mode::DevicesFile,
            records,
            unresolved_file_entries: file
                .entries
                .iter()
                .filter(|p| !state.devices.contains_key(*p))
                .cloned()
                .collect(),
            matching_work: 0,
        });
    }
    let compiled = policy
        .rules
        .iter()
        .map(|rule| ere::Regex::compile(&rule.pattern).map_err(|e| expression_error(e, None)))
        .collect::<Result<Vec<_>>>()?;
    let mut matchers = compiled
        .iter()
        .map(|r| r.matcher().map_err(|e| expression_error(e, None)))
        .collect::<Result<Vec<_>>>()?;
    let mut budget = ere::Budget::new(allowance).map_err(|e| expression_error(e, None))?;
    let mut records = Vec::new();
    for (path, device) in &state.devices {
        let mut allowed = true;
        let mut reason = Reason::NoFilterMatched;
        for (index, matcher) in matchers.iter_mut().enumerate() {
            if matcher
                .matches_line(path, &mut budget)
                .map_err(|e| expression_error(e, None))?
            {
                let action = policy.rules[index].action;
                allowed = action == Action::Accept;
                reason = Reason::FilterRule { index, action };
                break;
            }
        }
        records.push(Record {
            path: path.clone(),
            allowed,
            reason,
            metadata: allowed.then(|| metadata(device)),
        });
    }
    Ok(Report {
        mode: if policy.use_devices_file {
            Mode::FilterBecauseFileMissing
        } else {
            Mode::FilterSelected
        },
        records,
        unresolved_file_entries: vec![],
        matching_work: budget.spent(),
    })
}
