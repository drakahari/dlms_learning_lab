//! Bounded rule decisions over supplied decoded records.
//! No packet decoder, stream engine, operating-system action or host execution.
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    Syntax(String),
    Unsupported(String),
    Limit(String),
}
type Result<T> = std::result::Result<T, Fault>;
fn syntax(s: impl Into<String>) -> Fault {
    Fault::Syntax(s.into())
}
fn unsupported(s: impl Into<String>) -> Fault {
    Fault::Unsupported(s.into())
}
fn limit(s: impl Into<String>) -> Fault {
    Fault::Limit(s.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Tcp,
    Udp,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub protocol: Protocol,
    pub source: Ipv4Addr,
    pub source_port: u16,
    pub destination: Ipv4Addr,
    pub destination_port: u16,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub payload: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Address {
    Any,
    Network(u32, u8),
}
impl Address {
    fn mask(p: u8) -> u32 {
        if p == 0 { 0 } else { u32::MAX << (32 - p) }
    }
    fn parse(s: &str) -> Result<Self> {
        if s == "any" {
            return Ok(Self::Any);
        }
        if s.bytes().any(|b| b"![]$,:".contains(&b)) {
            return Err(unsupported(
                "Use any or one numeric IPv4 host/network; variables, lists, negation and IPv6 are outside this activity.",
            ));
        }
        let (host, prefix) = match s.split_once('/') {
            Some((h, p)) => (h, number(p, 32)? as u8),
            None => (s, 32),
        };
        let ip = host.parse::<Ipv4Addr>().map_err(|_| {
            syntax("Use a dotted-decimal IPv4 address and, optionally, /0 through /32.")
        })?;
        Ok(Self::Network(u32::from(ip) & Self::mask(prefix), prefix))
    }
    fn contains(self, ip: Ipv4Addr) -> bool {
        match self {
            Self::Any => true,
            Self::Network(n, p) => u32::from(ip) & Self::mask(p) == n,
        }
    }
}
fn number(s: &str, max: u32) -> Result<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(syntax("Use a decimal integer."));
    }
    if s.len() > 1 && s.starts_with('0') {
        return Err(unsupported(
            "Leading-zero number spellings are outside this activity; use ordinary decimal notation.",
        ));
    }
    s.parse::<u32>()
        .ok()
        .filter(|n| *n <= max)
        .ok_or_else(|| limit("Number exceeds this field's supported range."))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Port {
    Any,
    One(u16),
}
impl Port {
    fn parse(s: &str) -> Result<Self> {
        if s == "any" {
            return Ok(Self::Any);
        }
        if s.bytes().any(|b| b"![]$,:".contains(&b)) {
            return Err(unsupported(
                "Use any or one decimal port; ranges, lists, variables and negation are outside this activity.",
            ));
        }
        Ok(Self::One(number(s, 65535)? as u16))
    }
    fn contains(self, p: u16) -> bool {
        matches!(self, Self::Any) || self == Self::One(p)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Content {
    bytes: Vec<u8>,
    nocase: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    protocol: Protocol,
    source: Address,
    source_port: Port,
    destination: Address,
    destination_port: Port,
    bidirectional: bool,
    pub sid: u32,
    pub revision: Option<u32>,
    pub message: Option<String>,
    content: Vec<Content>,
}

fn options(s: &str) -> Result<Vec<&str>> {
    let mut out = vec![];
    let (mut begin, mut quoted, mut escaped) = (0, false, false);
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            ';' if !quoted => {
                let option = s[begin..i].trim();
                if option.is_empty() {
                    return Err(syntax(
                        "An empty option is not supported; keep one keyword:value; entry per option.",
                    ));
                }
                out.push(option);
                begin = i + 1;
            }
            '(' | ')' if !quoted => {
                return Err(syntax(
                    "Only one rule body is supported; check its parentheses.",
                ));
            }
            '\n' | '\r' if quoted => {
                return Err(unsupported(
                    "Literal line breaks inside quoted option strings are outside this activity.",
                ));
            }
            _ => {}
        }
    }
    if quoted || escaped {
        return Err(syntax("Close the quoted option value."));
    }
    if !s[begin..].trim().is_empty() {
        return Err(syntax("End every rule option with a semicolon."));
    }
    if out.len() > 7 {
        return Err(limit(
            "Use at most seven supported options in this one-rule activity.",
        ));
    }
    Ok(out)
}
fn quoted(s: &str, content: bool, max: usize) -> Result<(String, &str)> {
    let s = s.trim_start();
    if !s.starts_with('"') {
        return Err(syntax("Enclose the option text in double quotes."));
    }
    let mut text = String::new();
    let mut escaped = false;
    for (i, c) in s.char_indices().skip(1) {
        if escaped {
            if !['\\', '"', ';'].contains(&c) {
                return Err(unsupported(
                    "This activity supports only escaped backslash, double quote and semicolon in quoted text.",
                ));
            }
            text.push(c);
            escaped = false;
        } else {
            match c {
                '\\' => escaped = true,
                '"' => return Ok((text, &s[i + 1..])),
                ';' => return Err(syntax("Escape a semicolon inside quoted option text.")),
                '|' if content => {
                    return Err(unsupported(
                        "Hex/mixed-byte content is outside this ASCII-literal activity.",
                    ));
                }
                c if !(' '..='~').contains(&c) => {
                    return Err(unsupported("Use printable ASCII option text."));
                }
                _ => text.push(c),
            }
        }
        if text.len() > max {
            return Err(limit("Quoted option text exceeds the activity limit."));
        }
    }
    Err(syntax("Close the quoted option value."))
}
impl Rule {
    pub fn parse(input: &str) -> Result<Self> {
        if input.len() > 2048 {
            return Err(limit("Keep this rule file within 2 KiB."));
        }
        if !input.is_ascii()
            || input
                .bytes()
                .any(|b| b.is_ascii_control() && !b"\r\n\t".contains(&b))
        {
            return Err(unsupported(
                "Rule files use ASCII text, tabs and line breaks; binary/control input is outside this activity.",
            ));
        }
        // Only full-line # comments; never strip '#' inside a quoted value.
        let text = input
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        let (head, body) = text
            .trim()
            .split_once('(')
            .ok_or_else(|| syntax("Place the rule options in parentheses after the header."))?;
        let body = body
            .trim_end()
            .strip_suffix(')')
            .ok_or_else(|| syntax("End this single rule with a closing parenthesis."))?;
        let words = head.split_whitespace().collect::<Vec<_>>();
        let [
            action,
            protocol,
            source,
            source_port,
            direction,
            destination,
            destination_port,
        ] = words.as_slice()
        else {
            return Err(syntax(
                "Header format: alert tcp|udp SOURCE SOURCE_PORT -> DESTINATION DESTINATION_PORT.",
            ));
        };
        if *action != "alert" {
            return Err(unsupported(
                "Only alert rules are modeled; no drop, reject, log or inline action runs.",
            ));
        }
        let protocol = match *protocol {
            "tcp" => Protocol::Tcp,
            "udp" => Protocol::Udp,
            _ => {
                return Err(unsupported(
                    "Only tcp and udp headers over supplied IPv4 records are modeled.",
                ));
            }
        };
        let bidirectional = match *direction {
            "->" => false,
            "<>" => true,
            _ => {
                return Err(unsupported(
                    "Use -> or <>; other direction syntax is not modeled.",
                ));
            }
        };
        let mut rule = Self {
            protocol,
            source: Address::parse(source)?,
            source_port: Port::parse(source_port)?,
            destination: Address::parse(destination)?,
            destination_port: Port::parse(destination_port)?,
            bidirectional,
            sid: 0,
            revision: None,
            message: None,
            content: vec![],
        };
        let mut sid = false;
        for option in options(body)? {
            let (name, value) = option
                .split_once(':')
                .ok_or_else(|| syntax("Use keyword:value; for each supported option."))?;
            match name.trim() {
                "sid" if !sid => {
                    rule.sid = number(value.trim(), u32::MAX)?;
                    sid = true;
                }
                "rev" if rule.revision.is_none() => {
                    rule.revision = Some(number(value.trim(), u32::MAX)?)
                }
                "msg" if rule.message.is_none() => {
                    let (value, tail) = quoted(value, false, 160)?;
                    if !tail.trim().is_empty() {
                        return Err(syntax("Unexpected text after the quoted message."));
                    }
                    rule.message = Some(value);
                }
                "content" => {
                    if value.trim_start().starts_with('!') {
                        return Err(unsupported(
                            "Negated content is outside this activity; only positive literal checks are modeled.",
                        ));
                    }
                    if rule.content.len() == 4 {
                        return Err(limit("At most four literal content checks are modeled."));
                    }
                    let (value, tail) = quoted(value, true, 128)?;
                    if value.is_empty() {
                        return Err(unsupported(
                            "An empty content literal is outside this activity.",
                        ));
                    }
                    let tail = tail.trim();
                    let nocase = if tail.is_empty() {
                        false
                    } else if tail.strip_prefix(',').is_some_and(|s| s.trim() == "nocase") {
                        true
                    } else {
                        return Err(unsupported(
                            "Only the nocase content modifier is modeled; no offsets, relative cursors, service buffers or fast-pattern settings.",
                        ));
                    };
                    rule.content.push(Content {
                        bytes: value.into_bytes(),
                        nocase,
                    });
                }
                "sid" | "rev" | "msg" => {
                    return Err(syntax(
                        "A rule may not repeat sid, rev or msg in this activity.",
                    ));
                }
                other => {
                    return Err(unsupported(format!(
                        "Unsupported rule option: {other}. No stream, flow, regex, service, include or action behavior is silently assumed."
                    )));
                }
            }
        }
        if !sid || rule.sid == 0 {
            return Err(unsupported(
                "This activity requires a positive sid for its local rule. Snort permits omission; use local IDs starting at 1000000 when authoring your own rules.",
            ));
        }
        Ok(rule)
    }
    fn endpoints(&self, s: Ipv4Addr, sp: u16, d: Ipv4Addr, dp: u16) -> bool {
        self.source.contains(s)
            && self.source_port.contains(sp)
            && self.destination.contains(d)
            && self.destination_port.contains(dp)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    ProtocolMismatch,
    EndpointMismatch,
    MissingContent(usize),
    Alert,
}
fn spend(left: &mut usize) -> Result<()> {
    if *left == 0 {
        return Err(limit(
            "The shared event-replay work allowance was reached; no partial report is returned.",
        ));
    }
    *left -= 1;
    Ok(())
}
fn contains(payload: &[u8], needle: &Content, left: &mut usize) -> Result<bool> {
    if needle.bytes.len() > payload.len() {
        return Ok(false);
    }
    for at in 0..=payload.len() - needle.bytes.len() {
        let mut matched = true;
        for (a, b) in payload[at..].iter().zip(&needle.bytes) {
            spend(left)?;
            if if needle.nocase {
                !a.eq_ignore_ascii_case(b)
            } else {
                a != b
            } {
                matched = false;
                break;
            }
        }
        if matched {
            return Ok(true);
        }
    }
    Ok(false)
}
pub fn inspect(rule: &Rule, events: &[Event]) -> Result<Vec<Decision>> {
    if events.len() > 64 {
        return Err(limit("At most64supplied records are supported."));
    }
    // Validate the whole input before producing any observation.
    if events.iter().any(|e| {
        e.payload.len() > 4096
            || !e.payload.is_ascii()
            || e.payload
                .bytes()
                .any(|b| b.is_ascii_control() && !b"\r\n\t".contains(&b))
    }) {
        return Err(unsupported(
            "Use supplied ASCII payloads within4KiB; raw/binary packet decoding is not modeled.",
        ));
    }
    let mut left = 250_000;
    let mut out = vec![];
    for e in events {
        spend(&mut left)?;
        let mut decision = if e.protocol != rule.protocol {
            Decision::ProtocolMismatch
        } else if !(rule.endpoints(e.source, e.source_port, e.destination, e.destination_port)
            || (rule.bidirectional
                && rule.endpoints(e.destination, e.destination_port, e.source, e.source_port)))
        {
            Decision::EndpointMismatch
        } else {
            Decision::Alert
        };
        if decision == Decision::Alert {
            for (i, content) in rule.content.iter().enumerate() {
                if !contains(e.payload.as_bytes(), content, &mut left)? {
                    decision = Decision::MissingContent(i);
                    break;
                }
            }
        }
        out.push(decision);
    }
    Ok(out)
}
