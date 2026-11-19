//! Bounded Squid3.5-style access decisions over supplied requests; never a proxy.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::Ipv4Addr};

pub const MAX_CONFIG: usize = 8192;
pub const MAX_LINES: usize = 256;
pub const MAX_LINE: usize = 384;
pub const MAX_ACLS: usize = 32;
pub const MAX_VALUES: usize = 256;
pub const MAX_RULES: usize = 64;
pub const MAX_TERMS: usize = 16;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum FaultKind {
    Limit,
    Syntax,
    Unsupported,
    Invalid,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Fault {
    pub kind: FaultKind,
    pub line: Option<usize>,
    pub message: String,
}
fn fail(kind: FaultKind, line: Option<usize>, message: impl Into<String>) -> Fault {
    Fault {
        kind,
        line,
        message: message.into(),
    }
}
impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = self.line {
            write!(f, "Line {line}: ")?;
        }
        f.write_str(&self.message)
    }
}
impl std::error::Error for Fault {}
type Result<T> = std::result::Result<T, Fault>;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Allow,
    Deny,
}
impl Action {
    fn opposite(self) -> Self {
        match self {
            Self::Allow => Self::Deny,
            Self::Deny => Self::Allow,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub client: Ipv4Addr,
    #[serde(deserialize_with = "crate::cron::text::<_,20>")]
    pub method: String,
    pub destination_port: u16,
}
impl Request {
    pub fn validate(&self) -> Result<()> {
        if self.destination_port == 0 {
            return Err(fail(
                FaultKind::Invalid,
                None,
                "A supplied request needs a destination port from 1 to 65535.",
            ));
        }
        if !method(&self.method) {
            return Err(fail(
                FaultKind::Invalid,
                None,
                "A supplied request needs one case-sensitive HTTP method token, at most 20 ASCII characters.",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Network { base: u32, mask: u32 },
    Port { first: u16, last: u16 },
    Method(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Source,
    Port,
    Method,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Acl {
    kind: Kind,
    values: Vec<Value>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Term {
    name: String,
    negated: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Rule {
    line: usize,
    action: Action,
    terms: Vec<Term>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    acls: BTreeMap<String, Acl>,
    rules: Vec<Rule>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TermResult {
    pub name: String,
    pub negated: bool,
    pub matched: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RuleResult {
    pub line: usize,
    pub action: Action,
    pub matched: bool,
    pub terms: Vec<TermResult>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Decision {
    pub action: Action,
    pub matched_line: Option<usize>,
    pub explanation: String,
    pub rules: Vec<RuleResult>,
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
fn method(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 20
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
}
fn decimal<T: std::str::FromStr>(s: &str) -> Option<T> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        None
    } else {
        s.parse().ok()
    }
}
fn value(kind: Kind, s: &str, line: usize) -> Result<Value> {
    let invalid = || {
        fail(
            FaultKind::Syntax,
            Some(line),
            match kind {
                Kind::Source => {
                    "Use a dotted IPv4 address with an optional numeric /0 to /32 prefix; dotted masks, ranges and DNS names are outside this subset."
                }
                Kind::Port => {
                    "Use a destination port from 0 to 65535 or a closed ascending range such as 8000-8010."
                }
                Kind::Method => {
                    "Use a case-sensitive HTTP method token, at most 20 ASCII characters."
                }
            },
        )
    };
    match kind {
        Kind::Source => {
            let (ip, prefix) = match s.split_once('/') {
                Some((ip, p)) => (
                    ip,
                    decimal::<u8>(p).filter(|p| *p <= 32).ok_or_else(invalid)?,
                ),
                None => (s, 32),
            };
            let ip = ip.parse::<Ipv4Addr>().map_err(|_| invalid())?;
            let mask = if prefix == 0 {
                0
            } else {
                u32::MAX << (32 - prefix)
            };
            Ok(Value::Network {
                base: u32::from(ip) & mask,
                mask,
            })
        }
        Kind::Port => {
            let (first, last) = match s.split_once('-') {
                Some((a, b)) => (
                    decimal::<u16>(a).ok_or_else(invalid)?,
                    decimal::<u16>(b).ok_or_else(invalid)?,
                ),
                None => {
                    let p = decimal::<u16>(s).ok_or_else(invalid)?;
                    (p, p)
                }
            };
            if first > last {
                return Err(invalid());
            }
            Ok(Value::Port { first, last })
        }
        Kind::Method => {
            if !method(s) {
                return Err(invalid());
            }
            Ok(Value::Method(s.into()))
        }
    }
}
impl Policy {
    /// No include/files/expansion, regex, authentication, DNS or host operations.
    pub fn parse(text: &str) -> Result<Self> {
        if text.len() > MAX_CONFIG {
            return Err(fail(
                FaultKind::Limit,
                None,
                "Configuration exceeds 8192 bytes.",
            ));
        }
        if !text.is_ascii()
            || text
                .bytes()
                .any(|b| b.is_ascii_control() && !b"\n\r\t".contains(&b))
        {
            return Err(fail(
                FaultKind::Syntax,
                None,
                "This configuration subset accepts ASCII text, tabs and line endings only.",
            ));
        }
        let mut policy = Self {
            acls: BTreeMap::new(),
            rules: Vec::new(),
        };
        let mut values = 0;
        for (n, raw) in text.lines().enumerate() {
            let line = n + 1;
            if line > MAX_LINES || raw.len() > MAX_LINE {
                return Err(fail(
                    FaultKind::Limit,
                    Some(line),
                    "Use at most 256 lines and 384 bytes per line.",
                ));
            }
            let s = raw.trim();
            if s.is_empty() || s.starts_with('#') {
                continue;
            }
            if s.contains(['"', '\'', '\\', '#']) {
                return Err(fail(
                    FaultKind::Unsupported,
                    Some(line),
                    "Quoted files, escapes, continuation and inline comments are not supported; use separate comment lines.",
                ));
            }
            let words = s.split_ascii_whitespace().collect::<Vec<_>>();
            match words[0] {
                "acl" => {
                    if words.len() < 4 {
                        return Err(fail(
                            FaultKind::Syntax,
                            Some(line),
                            "An acl line needs a name, a type and at least one value.",
                        ));
                    }
                    let acl_name = words[1];
                    if !name(acl_name) || acl_name == "all" {
                        return Err(fail(
                            FaultKind::Syntax,
                            Some(line),
                            "Choose an ACL name of at most 48 letters, digits, underscores or hyphens; all is reserved.",
                        ));
                    }
                    let kind = match words[2] {
                        "src" => Kind::Source,
                        "port" => Kind::Port,
                        "method" => Kind::Method,
                        _ => {
                            return Err(fail(
                                FaultKind::Unsupported,
                                Some(line),
                                "Supported ACL types are src, port and method. Other types, authentication and DNS are outside this model.",
                            ));
                        }
                    };
                    values += words.len() - 3;
                    if values > MAX_VALUES {
                        return Err(fail(
                            FaultKind::Limit,
                            Some(line),
                            "Use at most 256 ACL values in total.",
                        ));
                    }
                    if !policy.acls.contains_key(acl_name) && policy.acls.len() == MAX_ACLS {
                        return Err(fail(
                            FaultKind::Limit,
                            Some(line),
                            "Use at most 32 named ACLs.",
                        ));
                    }
                    let entry = policy.acls.entry(acl_name.into()).or_insert_with(|| Acl {
                        kind,
                        values: Vec::new(),
                    });
                    if entry.kind != kind {
                        return Err(fail(
                            FaultKind::Syntax,
                            Some(line),
                            "The same ACL name cannot refer to different types.",
                        ));
                    }
                    for s in &words[3..] {
                        entry.values.push(value(kind, s, line)?);
                    }
                }
                "http_access" => {
                    if words.len() < 3 {
                        return Err(fail(
                            FaultKind::Syntax,
                            Some(line),
                            "An http_access line needs allow or deny and at least one ACL name.",
                        ));
                    }
                    if words.len() - 2 > MAX_TERMS || policy.rules.len() == MAX_RULES {
                        return Err(fail(
                            FaultKind::Limit,
                            Some(line),
                            "Use at most 64 access rules and 16 ACL tests per rule.",
                        ));
                    }
                    let action = match words[1] {
                        "allow" => Action::Allow,
                        "deny" => Action::Deny,
                        _ => {
                            return Err(fail(
                                FaultKind::Syntax,
                                Some(line),
                                "An access action must be allow or deny.",
                            ));
                        }
                    };
                    let mut terms = Vec::new();
                    for raw in &words[2..] {
                        let (negated, n) = if let Some(n) = raw.strip_prefix('!') {
                            (true, n)
                        } else {
                            (false, *raw)
                        };
                        if !name(n) || n != "all" && !policy.acls.contains_key(n) {
                            return Err(fail(
                                FaultKind::Syntax,
                                Some(line),
                                "Every access test must name all or an ACL already defined above the rule; one leading ! negates it.",
                            ));
                        }
                        terms.push(Term {
                            name: n.into(),
                            negated,
                        });
                    }
                    policy.rules.push(Rule {
                        line,
                        action,
                        terms,
                    });
                }
                _ => {
                    return Err(fail(
                        FaultKind::Unsupported,
                        Some(line),
                        "Only acl and http_access directives are supported. This workspace does not install Squid, bind a port, read files, authenticate users or configure a cache.",
                    ));
                }
            }
        }
        Ok(policy)
    }
    fn matches(&self, name: &str, r: &Request) -> bool {
        if name == "all" {
            return true;
        }
        self.acls.get(name).is_some_and(|a| {
            a.values.iter().any(|v| match v {
                Value::Network { base, mask } => u32::from(r.client) & mask == *base,
                Value::Port { first, last } => (*first..=*last).contains(&r.destination_port),
                Value::Method(m) => m == &r.method,
            })
        })
    }
    pub fn evaluate(&self, r: &Request) -> Result<Decision> {
        r.validate()?;
        let mut trace = Vec::new();
        for rule in &self.rules {
            let terms = rule
                .terms
                .iter()
                .map(|t| TermResult {
                    name: t.name.clone(),
                    negated: t.negated,
                    matched: self.matches(&t.name, r) ^ t.negated,
                })
                .collect::<Vec<_>>();
            let matched = terms.iter().all(|t| t.matched);
            trace.push(RuleResult {
                line: rule.line,
                action: rule.action,
                matched,
                terms,
            });
            if matched {
                return Ok(Decision {
                    action: rule.action,
                    matched_line: Some(rule.line),
                    explanation: format!(
                        "Line {} is the first rule whose ACL tests all match. Later rules are not reached.",
                        rule.line
                    ),
                    rules: trace,
                });
            }
        }
        let (action, explanation) = if let Some(last) = self.rules.last() {
            (
                last.action.opposite(),
                "No rule matched. Squid's implicit fallback is the opposite of the last rule; an explicit final deny all avoids relying on that fallback.",
            )
        } else {
            (
                Action::Deny,
                "No access rules are configured, so access is denied.",
            )
        };
        Ok(Decision {
            action,
            matched_line: None,
            explanation: explanation.into(),
            rules: trace,
        })
    }
}
#[cfg(test)]
mod tests;

/// Only virtual configuration text is stored; executable/parser internals never deserialize.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    saved: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    applied: String,
    generation: u32,
}
impl World {
    pub fn new(initial: &str) -> Result<Self> {
        Policy::parse(initial)?;
        Ok(Self {
            saved: initial.into(),
            applied: initial.into(),
            generation: 0,
        })
    }
    pub fn validate(&self) -> Result<()> {
        Self::text_shape(&self.saved)?;
        Policy::parse(&self.applied)?;
        if self.generation > 10_000 {
            return Err(fail(
                FaultKind::Limit,
                None,
                "The virtual configuration history exceeds this activity's limit; reset only this attempt to continue.",
            ));
        }
        Ok(())
    }
    fn text_shape(s: &str) -> Result<()> {
        if s.len() > MAX_CONFIG {
            return Err(fail(
                FaultKind::Limit,
                None,
                "Saved configuration exceeds 8192 bytes.",
            ));
        }
        if s.chars()
            .any(|c| c.is_control() && !['\n', '\r', '\t'].contains(&c))
        {
            return Err(fail(
                FaultKind::Syntax,
                None,
                "Configuration cannot contain nonprinting control characters.",
            ));
        }
        Ok(())
    }
    fn next(&self) -> Result<u32> {
        self.generation.checked_add(1).filter(|n|*n<=10_000).ok_or_else(||fail(FaultKind::Limit,None,"The virtual configuration history exceeds this activity's limit; reset only this attempt to continue."))
    }
    /// Syntax errors may be saved for diagnosis; nothing is activated by saving.
    pub fn save(&mut self, text: &str) -> Result<bool> {
        self.validate()?;
        Self::text_shape(text)?;
        if self.saved == text {
            return Ok(false);
        }
        let generation = self.next()?;
        self.saved = text.into();
        self.generation = generation;
        Ok(true)
    }
    pub fn validate_saved(&self) -> Result<()> {
        self.validate()?;
        Policy::parse(&self.saved)?;
        Ok(())
    }
    /// Atomic simulated activation: malformed/unsupported text preserves prior effective policy.
    pub fn apply_saved(&mut self) -> Result<bool> {
        self.validate_saved()?;
        if self.applied == self.saved {
            return Ok(false);
        }
        let generation = self.next()?;
        self.applied = self.saved.clone();
        self.generation = generation;
        Ok(true)
    }
    pub fn saved(&self) -> &str {
        &self.saved
    }
    pub fn applied(&self) -> &str {
        &self.applied
    }
    pub fn generation(&self) -> u32 {
        self.generation
    }
    pub fn effective_decision(&self, request: &Request) -> Result<Decision> {
        self.validate()?;
        Policy::parse(&self.applied)?.evaluate(request)
    }
    pub fn saved_decision(&self, request: &Request) -> Result<Decision> {
        self.validate()?;
        Policy::parse(&self.saved)?.evaluate(request)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Counterexample {
    pub request: Request,
    pub actual: Action,
    pub required: Action,
}
impl Policy {
    fn action(&self, r: &Request) -> Action {
        for rule in &self.rules {
            if rule
                .terms
                .iter()
                .all(|t| self.matches(&t.name, r) ^ t.negated)
            {
                return rule.action;
            }
        }
        self.rules
            .last()
            .map_or(Action::Deny, |r| r.action.opposite())
    }
    /// Exact for this finite Boolean language, not statistical sampling. Every ACL
    /// is constant within the source/port intervals and method classes considered.
    /// Excess complexity is an honest error, never assumed equivalence.
    pub fn compare(&self, required: &Self) -> Result<Option<Counterexample>> {
        let mut ips = std::collections::BTreeSet::from([0_u64, 1_u64 << 32]);
        let mut ports = std::collections::BTreeSet::from([1_u32, 65536_u32]);
        let mut methods = std::collections::BTreeSet::new();
        for policy in [self, required] {
            for a in policy.acls.values() {
                for v in &a.values {
                    match v {
                        Value::Network { base, mask } => {
                            ips.insert(u64::from(*base));
                            ips.insert(u64::from(*base) + u64::from(!*mask) + 1);
                        }
                        Value::Port { first, last } => {
                            if *first > 0 {
                                ports.insert(u32::from(*first));
                            }
                            ports.insert((u32::from(*last) + 1).max(1));
                        }
                        Value::Method(m) => {
                            methods.insert(m.clone());
                        }
                    }
                }
            }
        }
        for n in 0..=methods.len() {
            let candidate = format!("X{n}");
            if !methods.contains(&candidate) {
                methods.insert(candidate);
                break;
            }
        }
        const MAX_REGIONS: usize = 8192;
        let regions = (ips.len() - 1)
            .checked_mul(ports.len() - 1)
            .and_then(|n| n.checked_mul(methods.len()));
        if regions.is_none_or(|n| n > MAX_REGIONS) {
            return Err(fail(
                FaultKind::Limit,
                None,
                "This policy creates more than 8192 distinct request regions. Simplify redundant ACL values so the simulator can verify the full policy; no success has been assumed.",
            ));
        }
        let predicates = [self, required]
            .into_iter()
            .map(|policy| {
                policy
                    .rules
                    .iter()
                    .map(|r| {
                        r.terms
                            .iter()
                            .map(|t| policy.acls.get(&t.name).map_or(1, |a| a.values.len()))
                            .sum::<usize>()
                    })
                    .sum::<usize>()
            })
            .sum::<usize>()
            .max(1);
        if regions
            .unwrap()
            .checked_mul(predicates)
            .is_none_or(|work| work > 262_144)
        {
            return Err(fail(
                FaultKind::Limit,
                None,
                "This policy exceeds the bounded full-policy comparison work budget. Simplify redundant rules or values; no success has been assumed.",
            ));
        }
        let ip_edges = ips.into_iter().collect::<Vec<_>>();
        let mut first_mismatch = None;
        for edge in ip_edges.windows(2) {
            // Predicates are constant throughout the interval. Prefer an interior
            // address when available so feedback need not use a subnet boundary.
            let ip = if edge[1] - edge[0] > 1 {
                edge[0] + 1
            } else {
                edge[0]
            };
            for port in ports.iter().take(ports.len() - 1) {
                for method in &methods {
                    let request = Request {
                        client: Ipv4Addr::from(ip as u32),
                        method: method.clone(),
                        destination_port: *port as u16,
                    };
                    let actual = self.action(&request);
                    let required = required.action(&request);
                    if actual != required {
                        let witness = Counterexample {
                            request,
                            actual,
                            required,
                        };
                        if edge[1] - edge[0] > 1 {
                            return Ok(Some(witness));
                        }
                        first_mismatch.get_or_insert(witness);
                    }
                }
            }
        }
        Ok(first_mismatch)
    }
}
