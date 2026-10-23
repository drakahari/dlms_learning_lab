//! Bounded native configuration-document adapter. No IOS session or host I/O.
use crate::{acl, acl_routing as ar};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    net::Ipv4Addr,
};

pub const MAX_TEXT: usize = 65_536;
pub const MAX_LINES: usize = 1024;
pub const MAX_LINE: usize = 512;
pub const MAX_TOKENS: usize = 16;
pub const MAX_DRAFT_TOTAL: usize = 262_144;
pub const MAX_JSON: usize = 2_000_000;
pub const MAX_CASES: usize = 16;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Syntax,
    Unsupported,
    Resolution,
    Limit,
    InvalidData,
    FixedContext,
    Unapplied,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub category: Category,
    pub line: Option<usize>,
    pub message: String,
}
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(f, "Line {line}: ")?;
        }
        f.write_str(&self.message)
    }
}
impl std::error::Error for Diagnostic {}
fn err(category: Category, line: Option<usize>, message: impl Into<String>) -> Diagnostic {
    let mut message = message.into();
    if message.len() > 240 {
        let mut n = 237;
        while !message.is_char_boundary(n) {
            n -= 1;
        }
        message.truncate(n);
        message.push_str("...");
    }
    Diagnostic {
        category,
        line,
        message,
    }
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
fn kw(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}
fn empty() -> ar::DevicePolicy {
    ar::DevicePolicy {
        policy: acl::Policy {
            schema_version: 1,
            lists: BTreeMap::new(),
        },
        attachments: BTreeMap::new(),
    }
}
fn route<'a>(
    network: &'a ar::Network,
    router: &str,
) -> Result<&'a crate::ipv4_routing::Device, Diagnostic> {
    if !identifier(router) {
        return Err(err(
            Category::Resolution,
            None,
            "Invalid router identifier.",
        ));
    }
    if network.host_endpoints.contains_key(router) {
        return Err(err(
            Category::Unsupported,
            None,
            "Edit a transit router, not a declared host endpoint.",
        ));
    }
    network.topology.devices.get(router).ok_or_else(|| {
        err(
            Category::Resolution,
            None,
            "The selected router does not exist in this assignment.",
        )
    })
}
fn number(s: &str, line: usize) -> Result<u16, Diagnostic> {
    if s.bytes().any(|b| b.is_ascii_alphabetic()) {
        return Err(err(
            Category::Unsupported,
            Some(line),
            "Use a decimal integer; named services and other numeric notations are unsupported.",
        ));
    }
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(err(
            Category::Syntax,
            Some(line),
            "Use a decimal integer from 0 to 65535; named services and signed numbers are unsupported.",
        ));
    }
    s.parse().map_err(|_| {
        err(
            Category::Limit,
            Some(line),
            "The supported integer range is 0 to 65535.",
        )
    })
}
fn seq_id(id: &str) -> Option<u16> {
    let s = id.strip_prefix("seq-")?;
    let value = s.parse::<u16>().ok()?;
    (value > 0 && s == value.to_string()).then_some(value)
}
fn take<'a>(tokens: &[&'a str], at: &mut usize, line: usize) -> Result<&'a str, Diagnostic> {
    let value = tokens.get(*at).ok_or_else(|| {
        err(
            Category::Syntax,
            Some(line),
            "This line is incomplete; supply the missing address, port or action.",
        )
    })?;
    *at += 1;
    Ok(*value)
}
fn address(tokens: &[&str], at: &mut usize, line: usize) -> Result<acl::Wildcard, Diagnostic> {
    let first = take(tokens, at, line)?;
    if kw(first, "any") {
        return Ok(acl::Wildcard::any());
    }
    if kw(first, "host") {
        return Ok(acl::Wildcard::host(
            take(tokens, at, line)?.parse::<Ipv4Addr>().map_err(|_| {
                err(
                    Category::Syntax,
                    Some(line),
                    "host needs a dotted-decimal IPv4 address.",
                )
            })?,
        ));
    }
    let base = first.parse::<Ipv4Addr>().map_err(|_| {
        err(
            Category::Syntax,
            Some(line),
            "Use any, host ADDRESS, or ADDRESS WILDCARD.",
        )
    })?;
    let value=tokens.get(*at).ok_or_else(||err(Category::Unsupported,Some(line),"A bare IPv4 address is unsupported here. Use host ADDRESS or an explicit ADDRESS WILDCARD pair."))?;
    *at += 1;
    let wildcard=value.parse::<Ipv4Addr>().map_err(|_|err(Category::Unsupported,Some(line),"A bare IPv4 address is unsupported here. Use host ADDRESS or an explicit ADDRESS WILDCARD pair."))?;
    Ok(acl::Wildcard { base, wildcard })
}
fn ports(tokens: &[&str], at: &mut usize, line: usize) -> Result<acl::Ports, Diagnostic> {
    if tokens.get(*at).is_some_and(|s| kw(s, "eq")) {
        *at += 1;
        return Ok(acl::Ports::Eq {
            port: number(take(tokens, at, line)?, line)?,
        });
    }
    if tokens.get(*at).is_some_and(|s| kw(s, "range")) {
        *at += 1;
        let first = number(take(tokens, at, line)?, line)?;
        let last = number(take(tokens, at, line)?, line)?;
        if first > last {
            return Err(err(
                Category::Syntax,
                Some(line),
                "A port range starts with the smaller value and includes both ends.",
            ));
        }
        return Ok(acl::Ports::Range { first, last });
    }
    if tokens
        .get(*at)
        .is_some_and(|s| ["lt", "gt", "neq"].iter().any(|v| kw(s, v)))
    {
        return Err(err(
            Category::Unsupported,
            Some(line),
            "Only absent ports, eq and inclusive range are supported.",
        ));
    }
    Ok(acl::Ports::Any {})
}
enum Mode {
    Global,
    List {
        name: String,
        numbered: Option<bool>,
        header: usize,
    },
    Interface(String),
}
fn finish(mode: &Mode, config: &mut ar::DevicePolicy) -> Result<(), Diagnostic> {
    if let Mode::List {
        name,
        numbered,
        header,
    } = mode
    {
        let list = config
            .policy
            .lists
            .get_mut(name)
            .expect("parser owns declared list");
        if list.rules.is_empty() {
            return Err(err(
                Category::Unsupported,
                Some(*header),
                "Empty ACL sections are unsupported. Remove the section and its bindings to clear it.",
            ));
        }
        if *numbered == Some(true) {
            list.rules
                .sort_by_key(|r| seq_id(&r.id).expect("parser generated sequence"));
        }
    }
    Ok(())
}
/// Parses one entire router document. Never changes the network while typing/parsing.
pub fn parse_router(
    network: &ar::Network,
    router: &str,
    text: &str,
) -> Result<ar::DevicePolicy, Diagnostic> {
    network
        .validate()
        .map_err(|e| err(Category::InvalidData, None, e.message))?;
    let device = route(network, router)?;
    if text.len() > MAX_TEXT {
        return Err(err(
            Category::Limit,
            None,
            "A router document supports at most 64 KiB of UTF-8 text.",
        ));
    }
    let mut config = empty();
    let mut mode = Mode::Global;
    let mut seen_ports = BTreeSet::new();
    for (index, raw) in text.split_terminator('\n').enumerate() {
        let line = index + 1;
        if line > MAX_LINES || raw.len() > MAX_LINE {
            return Err(err(
                Category::Limit,
                Some(line),
                "The document supports 1024 lines and 512 bytes per line.",
            ));
        }
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        if raw.chars().any(|c| c.is_control() && c != '\t') {
            return Err(err(
                Category::Unsupported,
                Some(line),
                "Only spaces, tabs and ordinary line endings are supported as control characters.",
            ));
        }
        let source = raw.trim_matches([' ', '\t']);
        if source.is_empty() || source.starts_with('!') {
            continue;
        }
        let mut tokens = Vec::new();
        for token in source.split_ascii_whitespace() {
            if tokens.len() >= MAX_TOKENS {
                return Err(err(
                    Category::Limit,
                    Some(line),
                    "A configuration line supports at most 16 tokens.",
                ));
            }
            tokens.push(token);
        }
        if tokens.len() == 1 && kw(tokens[0], "exit") {
            if matches!(&mode, Mode::Global) {
                return Err(err(
                    Category::Syntax,
                    Some(line),
                    "There is no open section to exit.",
                ));
            }
            finish(&mode, &mut config)?;
            mode = Mode::Global;
            continue;
        }
        match &mut mode {
            Mode::Global => {
                if tokens.len() == 4 && kw(tokens[0], "ip") && kw(tokens[1], "access-list") {
                    let kind = if kw(tokens[2], "standard") {
                        acl::Kind::Standard
                    } else if kw(tokens[2], "extended") {
                        acl::Kind::Extended
                    } else {
                        return Err(err(
                            Category::Unsupported,
                            Some(line),
                            "Only standard and extended IPv4 access lists are supported.",
                        ));
                    };
                    let name = tokens[3];
                    if !identifier(name) {
                        return Err(err(
                            Category::Syntax,
                            Some(line),
                            "Use a 1 to 32 byte ACL identifier with letters, digits, dash, dot or underscore.",
                        ));
                    }
                    if config.policy.lists.len() >= acl::MAX_LISTS {
                        return Err(err(
                            Category::Limit,
                            Some(line),
                            "At most eight ACL sections are supported.",
                        ));
                    }
                    if config.policy.lists.contains_key(name) {
                        return Err(err(
                            Category::Syntax,
                            Some(line),
                            "This ACL section appears twice; combine its rules in one section.",
                        ));
                    }
                    config.policy.lists.insert(
                        name.into(),
                        acl::AccessList {
                            kind,
                            rules: Vec::new(),
                        },
                    );
                    mode = Mode::List {
                        name: name.into(),
                        numbered: None,
                        header: line,
                    };
                } else if tokens.len() == 2 && kw(tokens[0], "interface") {
                    let name = tokens[1];
                    if !device.interfaces.contains_key(name) {
                        return Err(err(
                            Category::Resolution,
                            Some(line),
                            "Use an interface name shown on the selected router's topology.",
                        ));
                    }
                    if !seen_ports.insert(name.to_owned()) {
                        return Err(err(
                            Category::Syntax,
                            Some(line),
                            "This interface section appears twice; combine its bindings.",
                        ));
                    }
                    config
                        .attachments
                        .insert(name.into(), ar::Attachment::default());
                    mode = Mode::Interface(name.into());
                } else {
                    return Err(err(
                        Category::Unsupported,
                        Some(line),
                        "Start an ip access-list standard|extended NAME or interface NAME section. Session commands and no are unsupported.",
                    ));
                }
            }
            Mode::Interface(name) => {
                if tokens.len() != 4 || !kw(tokens[0], "ip") || !kw(tokens[1], "access-group") {
                    return Err(err(
                        Category::Unsupported,
                        Some(line),
                        "An interface section supports ip access-group NAME in|out. Close it with exit before another section.",
                    ));
                }
                if !identifier(tokens[2]) {
                    return Err(err(Category::Syntax, Some(line), "Invalid bound ACL name."));
                }
                let a = config
                    .attachments
                    .get_mut(name.as_str())
                    .expect("parser owns interface");
                let value = if kw(tokens[3], "in") {
                    &mut a.inbound
                } else if kw(tokens[3], "out") {
                    &mut a.outbound
                } else {
                    return Err(err(
                        Category::Unsupported,
                        Some(line),
                        "Use in or out for this interface binding.",
                    ));
                };
                if value.is_some() {
                    return Err(err(
                        Category::Syntax,
                        Some(line),
                        "This interface direction already has a binding.",
                    ));
                }
                *value = Some(tokens[2].into());
            }
            Mode::List { name, numbered, .. } => {
                let list = config
                    .policy
                    .lists
                    .get_mut(name.as_str())
                    .expect("parser owns list");
                if list.rules.len() >= acl::MAX_RULES {
                    return Err(err(
                        Category::Limit,
                        Some(line),
                        "An ACL supports at most 64 rules.",
                    ));
                }
                let mut at = 0;
                let sequence = if tokens[0].bytes().all(|b| b.is_ascii_digit()) {
                    let n = number(take(&tokens, &mut at, line)?, line)?;
                    if n == 0 {
                        return Err(err(
                            Category::Syntax,
                            Some(line),
                            "Sequence numbers start at 1.",
                        ));
                    }
                    Some(n)
                } else {
                    None
                };
                if numbered.is_some_and(|mode| mode != sequence.is_some()) {
                    return Err(err(
                        Category::Unsupported,
                        Some(line),
                        "Use sequence numbers on every rule in this list, or on none; mixed mode is unsupported.",
                    ));
                }
                *numbered = Some(sequence.is_some());
                let id = sequence
                    .map(|n| format!("seq-{n}"))
                    .unwrap_or_else(|| format!("row-{}", list.rules.len() + 1));
                if list.rules.iter().any(|r| r.id == id) {
                    return Err(err(
                        Category::Syntax,
                        Some(line),
                        "This sequence number is already used in the list.",
                    ));
                }
                let action = take(&tokens, &mut at, line)?;
                let action = if kw(action, "permit") {
                    acl::Action::Permit
                } else if kw(action, "deny") {
                    acl::Action::Deny
                } else {
                    return Err(err(
                        Category::Unsupported,
                        Some(line),
                        "Use permit or deny. Close a section with exit before starting another.",
                    ));
                };
                let selector = match list.kind {
                    acl::Kind::Standard => acl::Selector::Standard {
                        source: address(&tokens, &mut at, line)?,
                    },
                    acl::Kind::Extended => {
                        let protocol = take(&tokens, &mut at, line)?;
                        let source = address(&tokens, &mut at, line)?;
                        let source_port = if kw(protocol, "tcp") || kw(protocol, "udp") {
                            ports(&tokens, &mut at, line)?
                        } else {
                            acl::Ports::Any {}
                        };
                        let destination = address(&tokens, &mut at, line)?;
                        let protocol = if kw(protocol, "tcp") || kw(protocol, "udp") {
                            let destination_port = ports(&tokens, &mut at, line)?;
                            if kw(protocol, "tcp") {
                                acl::Protocol::Tcp {
                                    source_port,
                                    destination_port,
                                }
                            } else {
                                acl::Protocol::Udp {
                                    source_port,
                                    destination_port,
                                }
                            }
                        } else if kw(protocol, "ip") {
                            acl::Protocol::Ip {}
                        } else if kw(protocol, "icmp") {
                            acl::Protocol::Icmp {}
                        } else {
                            return Err(err(
                                Category::Unsupported,
                                Some(line),
                                "Supported protocol selectors are ip, icmp, tcp and udp.",
                            ));
                        };
                        acl::Selector::Extended {
                            source,
                            destination,
                            protocol,
                        }
                    }
                };
                if at != tokens.len() {
                    return Err(err(
                        Category::Unsupported,
                        Some(line),
                        "Unexpected trailing syntax. Standard rules match source only; ip/icmp cannot carry ports; inline comments and other options are unsupported.",
                    ));
                }
                list.rules.push(acl::Rule {
                    id,
                    action,
                    selector,
                });
            }
        }
    }
    finish(&mode, &mut config)?;
    config
        .policy
        .validate()
        .map_err(|e| err(Category::InvalidData, None, e.message))?;
    for a in config.attachments.values() {
        for name in [&a.inbound, &a.outbound].into_iter().flatten() {
            if !config.policy.lists.contains_key(name) {
                return Err(err(
                    Category::Resolution,
                    None,
                    format!(
                        "Binding names missing ACL '{name}'. Restore its section or remove that binding."
                    ),
                ));
            }
        }
    }
    Ok(config)
}
fn append(out: &mut String, line: &str) -> Result<(), Diagnostic> {
    if out
        .len()
        .checked_add(line.len() + 1)
        .is_none_or(|n| n > MAX_TEXT)
    {
        return Err(err(
            Category::Limit,
            None,
            "The rendered router configuration exceeds 64 KiB.",
        ));
    }
    out.push_str(line);
    out.push('\n');
    Ok(())
}
fn format_address(a: acl::Wildcard) -> String {
    if a.wildcard == Ipv4Addr::UNSPECIFIED {
        format!("host {}", a.base)
    } else if a.base == Ipv4Addr::UNSPECIFIED && a.wildcard == Ipv4Addr::BROADCAST {
        "any".into()
    } else {
        format!("{} {}", a.base, a.wildcard)
    }
}
fn format_ports(p: acl::Ports) -> String {
    match p {
        acl::Ports::Any {} => String::new(),
        acl::Ports::Eq { port } => format!(" eq {port}"),
        acl::Ports::Range { first, last } => format!(" range {first} {last}"),
    }
}
fn format_selector(s: acl::Selector) -> String {
    match s {
        acl::Selector::Standard { source } => format_address(source),
        acl::Selector::Extended {
            source,
            destination,
            protocol,
        } => {
            let (p, sp, dp) = match protocol {
                acl::Protocol::Ip {} => ("ip", String::new(), String::new()),
                acl::Protocol::Icmp {} => ("icmp", String::new(), String::new()),
                acl::Protocol::Tcp {
                    source_port,
                    destination_port,
                } => (
                    "tcp",
                    format_ports(source_port),
                    format_ports(destination_port),
                ),
                acl::Protocol::Udp {
                    source_port,
                    destination_port,
                } => (
                    "udp",
                    format_ports(source_port),
                    format_ports(destination_port),
                ),
            };
            format!(
                "{p} {}{sp} {}{dp}",
                format_address(source),
                format_address(destination)
            )
        }
    }
}
fn render_config(config: &ar::DevicePolicy) -> Result<String, Diagnostic> {
    let mut out = String::new();
    for (name, list) in &config.policy.lists {
        append(
            &mut out,
            &format!(
                "ip access-list {} {name}",
                match list.kind {
                    acl::Kind::Standard => "standard",
                    acl::Kind::Extended => "extended",
                }
            ),
        )?;
        let numbered = list.rules.first().and_then(|r| seq_id(&r.id)).is_some();
        let mut previous = 0;
        for (i, rule) in list.rules.iter().enumerate() {
            let prefix = if numbered {
                let n = seq_id(&rule.id).filter(|n| *n > previous).ok_or_else(|| {
                    err(
                        Category::Unsupported,
                        None,
                        "The native renderer needs ascending compiler-generated seq-N identities.",
                    )
                })?;
                previous = n;
                format!("{n} ")
            } else {
                if rule.id != format!("row-{}", i + 1) {
                    return Err(err(
                        Category::Unsupported,
                        None,
                        "Opaque or reordered core rule IDs cannot be rendered by this native editor subset.",
                    ));
                }
                String::new()
            };
            append(
                &mut out,
                &format!(
                    " {prefix}{} {}",
                    match rule.action {
                        acl::Action::Permit => "permit",
                        acl::Action::Deny => "deny",
                    },
                    format_selector(rule.selector)
                ),
            )?;
        }
        append(&mut out, "exit")?;
    }
    for (name, a) in &config.attachments {
        append(&mut out, &format!("interface {name}"))?;
        for (direction, acl) in [("in", &a.inbound), ("out", &a.outbound)] {
            if let Some(acl) = acl {
                append(&mut out, &format!(" ip access-group {acl} {direction}"))?;
            }
        }
        append(&mut out, "exit")?;
    }
    Ok(out)
}
pub fn render_router(network: &ar::Network, router: &str) -> Result<String, Diagnostic> {
    network
        .validate()
        .map_err(|e| err(Category::InvalidData, None, e.message))?;
    route(network, router)?;
    network
        .devices
        .get(router)
        .map(render_config)
        .unwrap_or_else(|| Ok(String::new()))
}

struct Id(String);
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Name;
        impl Visitor<'_> for Name {
            type Value = Id;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a bounded identifier")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Id, E> {
                if !identifier(s) {
                    return Err(E::custom("Invalid editor identifier"));
                }
                Ok(Id(s.into()))
            }
        }
        d.deserialize_str(Name)
    }
}
fn id<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Id::deserialize(d).map(|v| v.0)
}
struct TextSeed {
    max: usize,
}
impl Visitor<'_> for TextSeed {
    type Value = String;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("bounded raw draft text")
    }
    fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
        if s.len() > self.max {
            return Err(E::custom("Draft text bound exceeded"));
        }
        Ok(s.into())
    }
}
impl<'de> DeserializeSeed<'de> for TextSeed {
    type Value = String;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
        d.deserialize_str(self)
    }
}
fn draft_map<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, String>, D::Error> {
    struct DraftMap;
    impl<'de> Visitor<'de> for DraftMap {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most12 unique bounded router drafts")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            let mut bytes = 0;
            while let Some(Id(key)) = a.next_key()? {
                if out.len() >= 12 || out.contains_key(&key) {
                    return Err(A::Error::custom("Duplicate router draft or limit"));
                }
                let text = a.next_value_seed(TextSeed {
                    max: MAX_TEXT.min(MAX_DRAFT_TOTAL - bytes),
                })?;
                bytes += text.len();
                out.insert(key, text);
            }
            Ok(out)
        }
    }
    d.deserialize_map(DraftMap)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Drafts {
    pub schema_version: u32,
    #[serde(deserialize_with = "draft_map")]
    pub routers: BTreeMap<String, String>,
}
impl Default for Drafts {
    fn default() -> Self {
        Self {
            schema_version: 1,
            routers: BTreeMap::new(),
        }
    }
}
impl Drafts {
    pub fn validate(&self, network: &ar::Network) -> Result<(), Diagnostic> {
        network
            .validate()
            .map_err(|e| err(Category::InvalidData, None, e.message))?;
        if self.schema_version != 1 || self.routers.len() > 12 {
            return Err(err(
                Category::InvalidData,
                None,
                "Unsupported draft version or router count.",
            ));
        }
        let mut total = 0;
        for (router, text) in &self.routers {
            route(network, router)?;
            if text.len() > MAX_TEXT {
                return Err(err(Category::Limit, None, "A router draft exceeds 64 KiB."));
            }
            total += text.len();
            if total > MAX_DRAFT_TOTAL {
                return Err(err(
                    Category::Limit,
                    None,
                    "Combined router drafts exceed 256 KiB.",
                ));
            }
        }
        Ok(())
    }
    pub fn from_json(bytes: &[u8], network: &ar::Network) -> Result<Self, Diagnostic> {
        if bytes.len() > MAX_JSON {
            return Err(err(Category::Limit, None, "Draft JSON exceeds 2 MB."));
        }
        let drafts: Self = serde_json::from_slice(bytes).map_err(|_| {
            err(
                Category::InvalidData,
                None,
                "Malformed draft data, duplicate keys or text bounds.",
            )
        })?;
        drafts.validate(network)?;
        Ok(drafts)
    }
    pub fn matches(&self, network: &ar::Network) -> Result<(), Diagnostic> {
        self.validate(network)?;
        for (router, text) in &self.routers {
            let parsed = parse_router(network, router, text)?;
            let matches = network
                .devices
                .get(router)
                .map_or(parsed == empty(), |existing| *existing == parsed);
            if !matches {
                return Err(err(
                    Category::Unapplied,
                    None,
                    format!(
                        "{router} has unapplied configuration edits. Apply/check the draft before current completion."
                    ),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expectation {
    Allowed {},
    Denied { expected: DenialExpectation },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DenialExpectation {
    pub at: ar::Location,
    /// None accepts either explicit or implicit denial at the exact boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<ar::ExpectedCause>,
}
impl DenialExpectation {
    fn valid_for(&self, network: &ar::Network) -> bool {
        identifier(&self.at.device)
            && identifier(&self.at.interface)
            && network
                .topology
                .devices
                .get(&self.at.device)
                .is_some_and(|d| d.interfaces.contains_key(&self.at.interface))
            && match &self.cause {
                Some(ar::ExpectedCause::Explicit { rule: Some(id) }) => identifier(id),
                _ => true,
            }
    }
    fn guarded(&self, observation: &ar::Inspection, positives: &[&ar::Inspection]) -> bool {
        let ar::Outcome::Denied { denial, .. } = &observation.actual else {
            return false;
        };
        let cause = self.cause.clone().unwrap_or(match &denial.decision.cause {
            acl::Cause::ImplicitDeny => ar::ExpectedCause::Implicit {},
            acl::Cause::Matched { .. } => ar::ExpectedCause::Explicit { rule: None },
        });
        ar::guarded_denial(
            observation,
            &ar::ExpectedDenial {
                at: self.at.clone(),
                cause,
            },
            positives,
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "id")]
    pub id: String,
    #[serde(deserialize_with = "label")]
    pub label: String,
    pub flow: ar::Flow,
    pub expectation: Expectation,
}
fn label<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    struct Label;
    impl Visitor<'_> for Label {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a short plain-language label")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.is_empty()
                || s.len() > 320
                || s.chars().count() > 80
                || s.chars().any(char::is_control)
            {
                return Err(E::custom(
                    "Label exceeds80 characters/320bytes or contains controls",
                ));
            }
            Ok(s.into())
        }
    }
    d.deserialize_str(Label)
}
fn cases<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Case>, D::Error> {
    struct Cases;
    impl<'de> Visitor<'de> for Cases {
        type Value = Vec<Case>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("2 to16 unique ACL cases")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            let mut seen = BTreeSet::new();
            while let Some(case) = a.next_element::<Case>()? {
                if out.len() >= MAX_CASES || !seen.insert(case.id.clone()) {
                    return Err(A::Error::custom("Case count or duplicate ID"));
                }
                out.push(case);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Cases)
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub schema_version: u32,
    pub initial: ar::Network,
    #[serde(deserialize_with = "cases")]
    pub cases: Vec<Case>,
}
fn crosses(observation: &ar::Inspection, at: &ar::Location) -> bool {
    matches!(&observation.actual,ar::Outcome::Delivered{events,..}if events.iter().any(|e|matches!(e,ar::Event::Boundary{at:hit,..}if hit==at)))
}
impl Definition {
    pub fn from_json(bytes: &[u8]) -> Result<Self, Diagnostic> {
        if bytes.len() > MAX_JSON {
            return Err(err(Category::Limit, None, "Definition JSON exceeds 2 MB."));
        }
        let definition: Self = serde_json::from_slice(bytes).map_err(|_| {
            err(
                Category::InvalidData,
                None,
                "Malformed definition, duplicate IDs or unsupported fields.",
            )
        })?;
        definition.validate()?;
        Ok(definition)
    }
    pub fn validate(&self) -> Result<(), Diagnostic> {
        if self.schema_version != 1 || !(2..=MAX_CASES).contains(&self.cases.len()) {
            return Err(err(
                Category::InvalidData,
                None,
                "An ACL definition needs version1 and 2 to16 cases.",
            ));
        }
        self.initial
            .validate()
            .map_err(|e| err(Category::InvalidData, None, e.message))?;
        let mut ids = BTreeSet::new();
        let mut positives = 0;
        let mut negatives = 0;
        for case in &self.cases {
            if !identifier(&case.id)
                || !ids.insert(case.id.as_str())
                || case.label.len() > 320
                || case.label.chars().count() > 80
                || case.label.is_empty()
                || case.label.chars().any(char::is_control)
            {
                return Err(err(
                    Category::InvalidData,
                    None,
                    "Case IDs must be unique; labels need 1 to80 ordinary characters and at most320 UTF-8 bytes.",
                ));
            }
            case.flow
                .validate()
                .map_err(|e| err(Category::InvalidData, None, e.message))?;
            match &case.expectation {
                Expectation::Allowed {} => positives += 1,
                Expectation::Denied { expected } => {
                    negatives += 1;
                    if !expected.valid_for(&self.initial)
                        || self
                            .initial
                            .host_endpoints
                            .contains_key(&expected.at.device)
                    {
                        return Err(err(
                            Category::InvalidData,
                            None,
                            "A denial location must identify a supplied transit-router interface.",
                        ));
                    }
                    if let Some(ar::ExpectedCause::Explicit { rule: Some(id) }) = &expected.cause
                        && seq_id(id).is_none()
                    {
                        return Err(err(
                            Category::InvalidData,
                            None,
                            "A prescribed rule identity must name an explicit seq-N row, not an opaque or unnumbered identity.",
                        ));
                    }
                }
            }
        }
        if positives == 0 || negatives == 0 {
            return Err(err(
                Category::InvalidData,
                None,
                "Require both allowed traffic and at least one denied flow.",
            ));
        }
        for config in self.initial.devices.values() {
            render_config(config)?;
        }
        // Count/scalar/network validation precedes the fixed no-filter baseline clone.
        let baseline = ar::Network {
            schema_version: 1,
            topology: self.initial.topology.clone(),
            host_endpoints: self.initial.host_endpoints.clone(),
            devices: BTreeMap::new(),
        };
        let mut observations = Vec::new();
        for case in &self.cases {
            let result = baseline
                .inspect(&case.flow)
                .map_err(|e| err(Category::InvalidData, None, e.message))?;
            if !result.delivered() {
                return Err(err(
                    Category::InvalidData,
                    None,
                    "Every authored flow needs no-ACL endpoint delivery in the supplied topology.",
                ));
            }
            observations.push(result);
        }
        for (i, case) in self.cases.iter().enumerate() {
            if let Expectation::Denied { expected } = &case.expectation
                && (!crosses(&observations[i], &expected.at)
                    || !self.cases.iter().zip(&observations).any(|(c, o)| {
                        matches!(c.expectation, Expectation::Allowed {}) && crosses(o, &expected.at)
                    }))
            {
                return Err(err(
                    Category::InvalidData,
                    None,
                    "Each negative location must be crossed by that baseline and a required positive baseline.",
                ));
            }
        }
        Ok(())
    }
    fn current(&self, network: &ar::Network) -> Result<(), Diagnostic> {
        network
            .validate()
            .map_err(|e| err(Category::InvalidData, None, e.message))?;
        self.validate()?;
        if network.topology != self.initial.topology
            || network.host_endpoints != self.initial.host_endpoints
        {
            return Err(err(
                Category::FixedContext,
                None,
                "This ACL assignment keeps all supplied topology, addresses, routes, operational flags and host roles fixed.",
            ));
        }
        for config in network.devices.values() {
            render_config(config)?;
        }
        Ok(())
    }
}
/// Returns a new bounded candidate; the input and all other router policies are untouched.
pub fn apply_router(
    definition: &Definition,
    current: &ar::Network,
    router: &str,
    text: &str,
) -> Result<ar::Network, Diagnostic> {
    definition.current(current)?;
    let config = parse_router(current, router, text)?;
    let mut lists = config.policy.lists.len();
    let mut rules = config
        .policy
        .lists
        .values()
        .map(|l| l.rules.len())
        .sum::<usize>();
    for (name, other) in &current.devices {
        if name != router {
            lists += other.policy.lists.len();
            rules += other
                .policy
                .lists
                .values()
                .map(|l| l.rules.len())
                .sum::<usize>();
        }
    }
    if lists > acl::MAX_LISTS || rules > acl::MAX_LISTS * acl::MAX_RULES {
        return Err(err(
            Category::Limit,
            None,
            "The candidate network exceeds eight lists or 512 rules.",
        ));
    }
    let mut candidate = current.clone();
    if config.policy.lists.is_empty() && config.attachments.is_empty() {
        candidate.devices.remove(router);
    } else {
        candidate.devices.insert(router.into(), config);
    }
    candidate
        .validate()
        .map_err(|e| err(Category::InvalidData, None, e.message))?;
    Ok(candidate)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseReport {
    pub id: String,
    pub label: String,
    pub passed: bool,
    pub observation: ar::Inspection,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub complete: bool,
    pub cases: Vec<CaseReport>,
}
/// Fresh evaluation only: there is no parameter accepting a cached Inspection/Report/pass flag.
pub fn grade(
    definition: &Definition,
    current: &ar::Network,
    drafts: &Drafts,
) -> Result<Report, Diagnostic> {
    definition.current(current)?;
    drafts.matches(current)?;
    let mut observations = Vec::new();
    for case in &definition.cases {
        observations.push(
            current
                .inspect(&case.flow)
                .map_err(|e| err(Category::InvalidData, None, e.message))?,
        );
    }
    let positives: Vec<_> = definition
        .cases
        .iter()
        .zip(&observations)
        .filter_map(|(case, o)| matches!(case.expectation, Expectation::Allowed {}).then_some(o))
        .collect();
    let passes: Vec<_> = definition
        .cases
        .iter()
        .zip(&observations)
        .map(|(case, o)| match &case.expectation {
            Expectation::Allowed {} => o.delivered(),
            Expectation::Denied { expected } => expected.guarded(o, &positives),
        })
        .collect();
    let complete = passes.iter().all(|p| *p);
    let cases = definition
        .cases
        .iter()
        .zip(observations)
        .zip(passes)
        .map(|((case, observation), passed)| CaseReport {
            id: case.id.clone(),
            label: case.label.clone(),
            passed,
            observation,
        })
        .collect();
    Ok(Report { complete, cases })
}
