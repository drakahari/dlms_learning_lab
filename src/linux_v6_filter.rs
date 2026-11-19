//! Numeric IPv6 filter/header predicates. No host I/O, kernel, routing or connection state.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv6Addr};
pub const MAX_RULES: usize = 96;
pub use crate::linux_filter::{Chain, Class, Decision, Fault, Output, Verdict};
fn parse_chain(s: &str) -> Result<Chain, Fault> {
    match s {
        "INPUT" => Ok(Chain::Input),
        "FORWARD" => Ok(Chain::Forward),
        "OUTPUT" => Ok(Chain::Output),
        _ => err(
            Class::Unsupported,
            "Only INPUT, FORWARD and OUTPUT are supported.",
        ),
    }
}
fn parse_verdict(s: &str) -> Result<Verdict, Fault> {
    match s {
        "ACCEPT" => Ok(Verdict::Accept),
        "DROP" => Ok(Verdict::Drop),
        _ => err(
            Class::Unsupported,
            "Choose ACCEPT or DROP; other targets/custom chains are outside this profile.",
        ),
    }
}
fn verdict_name(v: Verdict) -> &'static str {
    match v {
        Verdict::Accept => "ACCEPT",
        Verdict::Drop => "DROP",
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp6,
}
impl Protocol {
    fn name(self) -> &'static str {
        match self {
            Self::Tcp => "tcp",
            Self::Udp => "udp",
            Self::Icmp6 => "ipv6-icmp",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cidr {
    pub network: Ipv6Addr,
    pub prefix: u8,
}
impl Cidr {
    fn mask(prefix: u8) -> Option<u128> {
        match prefix {
            0 => Some(0),
            1..=128 => Some(u128::MAX << (128 - prefix)),
            _ => None,
        }
    }
    pub fn parse(s: &str) -> Result<Self, Fault> {
        let (a, p) = s.split_once('/').unwrap_or((s, "128"));
        let ip = a.parse::<Ipv6Addr>().map_err(|_| {
            fault(
                Class::Syntax,
                "Use a numeric IPv6 address or CIDR; DNS names are not resolved.",
            )
        })?;
        if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
            return err(Class::Syntax, "Use an IPv6 prefix from 0 to 128.");
        }
        let prefix = p
            .parse::<u8>()
            .map_err(|_| fault(Class::Syntax, "Use an IPv6 prefix from 0 to 128."))?;
        let mask = Self::mask(prefix)
            .ok_or_else(|| fault(Class::Syntax, "Use an IPv6 prefix from 0 to 128."))?;
        Ok(Self {
            network: Ipv6Addr::from(u128::from(ip) & mask),
            prefix,
        })
    }
    pub fn valid(self) -> bool {
        Self::mask(self.prefix)
            .is_some_and(|m| u128::from(self.network) & m == u128::from(self.network))
    }
    pub fn contains(self, ip: Ipv6Addr) -> bool {
        Self::mask(self.prefix).is_some_and(|m| u128::from(ip) & m == u128::from(self.network))
    }
    fn text(self) -> String {
        format!("{}/{}", self.network, self.prefix)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub source: Option<Cidr>,
    pub destination: Option<Cidr>,
    #[serde(deserialize_with = "optional_device")]
    pub ingress: Option<String>,
    #[serde(deserialize_with = "optional_device")]
    pub egress: Option<String>,
    pub protocol: Option<Protocol>,
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
    pub icmp_type: Option<u8>,
    pub icmp_code: Option<u8>,
    pub target: Verdict,
}
fn optional_device<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Name(#[serde(deserialize_with = "crate::cron::text::<_,15>")] String);
    Option::<Name>::deserialize(d).map(|v| v.map(|n| n.0))
}
fn device(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 15
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b))
}
impl Rule {
    pub(crate) fn valid(&self, c: Chain) -> bool {
        self.source.is_none_or(Cidr::valid)
            && self.destination.is_none_or(Cidr::valid)
            && self.ingress.as_deref().is_none_or(device)
            && self.egress.as_deref().is_none_or(device)
            && !(c == Chain::Output && self.ingress.is_some())
            && !(c == Chain::Input && self.egress.is_some())
            && ((self.source_port.is_none() && self.destination_port.is_none())
                || matches!(self.protocol, Some(Protocol::Tcp | Protocol::Udp)))
            && ((self.icmp_type.is_none() && self.icmp_code.is_none())
                || self.protocol == Some(Protocol::Icmp6))
            && (self.icmp_code.is_none() || self.icmp_type.is_some())
    }
    pub fn matches(&self, p: &Packet) -> bool {
        self.source.is_none_or(|n| n.contains(p.source))
            && self.destination.is_none_or(|n| n.contains(p.destination))
            && self
                .ingress
                .as_deref()
                .is_none_or(|i| p.ingress.as_deref() == Some(i))
            && self
                .egress
                .as_deref()
                .is_none_or(|o| p.egress.as_deref() == Some(o))
            && self.protocol.is_none_or(|v| v == p.protocol)
            && self.source_port.is_none_or(|v| p.source_port == Some(v))
            && self
                .destination_port
                .is_none_or(|v| p.destination_port == Some(v))
            && self.icmp_type.is_none_or(|v| p.icmp_type == Some(v))
            && self.icmp_code.is_none_or(|v| p.icmp_code == Some(v))
    }
    fn text(&self, c: Chain) -> String {
        let mut s = format!("-A {}", c.name());
        for (k, v) in [
            ("-s", self.source.map(Cidr::text)),
            ("-d", self.destination.map(Cidr::text)),
            ("-i", self.ingress.clone()),
            ("-o", self.egress.clone()),
            ("-p", self.protocol.map(|p| p.name().into())),
            ("--sport", self.source_port.map(|p| p.to_string())),
            ("--dport", self.destination_port.map(|p| p.to_string())),
        ] {
            if let Some(v) = v {
                s.push_str(&format!(" {k} {v}"));
            }
        }
        if let Some(t) = self.icmp_type {
            s.push_str(&format!(" --icmpv6-type {t}"));
            if let Some(c) = self.icmp_code {
                s.push_str(&format!("/{c}"));
            }
        }
        s.push_str(&format!(" -j {}", verdict_name(self.target)));
        s
    }
}
/// Path is derived by the separate router evaluator, never chosen by a learner to bypass a gate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packet {
    pub source: Ipv6Addr,
    pub destination: Ipv6Addr,
    pub ingress: Option<String>,
    pub egress: Option<String>,
    pub protocol: Protocol,
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
    pub icmp_type: Option<u8>,
    pub icmp_code: Option<u8>,
}
impl Packet {
    fn valid(&self) -> bool {
        self.ingress.as_deref().is_none_or(device)
            && self.egress.as_deref().is_none_or(device)
            && match self.protocol {
                Protocol::Icmp6 => {
                    self.source_port.is_none()
                        && self.destination_port.is_none()
                        && self.icmp_type.is_some()
                        && self.icmp_code.is_some()
                }
                Protocol::Tcp | Protocol::Udp => {
                    self.source_port.is_some()
                        && self.destination_port.is_some()
                        && self.icmp_type.is_none()
                        && self.icmp_code.is_none()
                }
            }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainRules {
    pub policy: Verdict,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Rule,96>")]
    pub rules: Vec<Rule>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub input: ChainRules,
    pub forward: ChainRules,
    pub output: ChainRules,
}
const CHAINS: [Chain; 3] = [Chain::Input, Chain::Forward, Chain::Output];
fn fault(class: Class, message: &str) -> Fault {
    Fault {
        class,
        message: message.into(),
    }
}
fn err<T>(class: Class, message: &str) -> Result<T, Fault> {
    Err(fault(class, message))
}
impl Table {
    pub fn new(policy: Verdict) -> Self {
        let c = ChainRules {
            policy,
            rules: vec![],
        };
        Self {
            input: c.clone(),
            forward: c.clone(),
            output: c,
        }
    }
    pub(crate) fn chain(&self, c: Chain) -> &ChainRules {
        match c {
            Chain::Input => &self.input,
            Chain::Forward => &self.forward,
            Chain::Output => &self.output,
        }
    }
    fn chain_mut(&mut self, c: Chain) -> &mut ChainRules {
        match c {
            Chain::Input => &mut self.input,
            Chain::Forward => &mut self.forward,
            Chain::Output => &mut self.output,
        }
    }
    pub fn validate(&self) -> Result<(), Fault> {
        if CHAINS
            .iter()
            .map(|c| self.chain(*c).rules.len())
            .sum::<usize>()
            > MAX_RULES
        {
            return err(
                Class::Limit,
                "Use at most 96 filter rules across all chains.",
            );
        }
        if !CHAINS
            .iter()
            .all(|c| self.chain(*c).rules.iter().all(|r| r.valid(*c)))
        {
            return err(
                Class::Context,
                "The filter contains an invalid numeric prefix, device, protocol/port or chain direction.",
            );
        }
        Ok(())
    }
    pub fn evaluate(&self, c: Chain, p: &Packet) -> Result<Decision, Fault> {
        self.validate()?;
        let path = match c {
            Chain::Input => p.ingress.is_some() && p.egress.is_none(),
            Chain::Output => p.ingress.is_none() && p.egress.is_some(),
            Chain::Forward => p.ingress.is_some() && p.egress.is_some(),
        };
        if !p.valid() || !path {
            return err(
                Class::Context,
                "The supplied synthetic packet or chain direction is invalid.",
            );
        }
        let chain = self.chain(c);
        Ok(chain.rules.iter().position(|r| r.matches(p)).map_or(
            Decision {
                verdict: chain.policy,
                rule: None,
            },
            |i| Decision {
                verdict: chain.rules[i].target,
                rule: Some(i + 1),
            },
        ))
    }
    pub fn listing(&self, which: Option<Chain>) -> String {
        let chosen = CHAINS
            .into_iter()
            .filter(|c| which.is_none_or(|w| *c == w))
            .collect::<Vec<_>>();
        let mut s = String::new();
        for c in &chosen {
            s.push_str(&format!(
                "-P {} {}\n",
                c.name(),
                verdict_name(self.chain(*c).policy)
            ));
        }
        for c in chosen {
            for r in &self.chain(c).rules {
                s.push_str(&r.text(c));
                s.push('\n');
            }
        }
        s
    }
    /// Parse against a scratch table; failure never leaves a partially edited policy.
    pub fn command(&mut self, line: &str) -> Result<Output, Fault> {
        self.validate()?;
        let mut scratch = self.clone();
        let text = scratch.apply(line)?;
        scratch.validate()?;
        if text.len() > 32 * 1024 {
            return err(
                Class::Limit,
                "The modeled filter output exceeded 32 KiB; the previous policy was retained.",
            );
        }
        let changed = scratch != *self;
        *self = scratch;
        Ok(Output { text, changed })
    }
    fn apply(&mut self, line: &str) -> Result<String, Fault> {
        let owned = words(line)?;
        let mut w = owned.iter().map(String::as_str).collect::<Vec<_>>();
        if w.first() != Some(&"ip6tables") {
            return err(
                Class::Unsupported,
                "This first profile models IPv6 ip6tables filter commands only; no host command ran.",
            );
        }
        w.remove(0);
        let mut i = 0;
        let mut table_seen = false;
        while i < w.len() {
            if matches!(w[i], "-t" | "--table") {
                if table_seen {
                    return err(Class::Syntax, "Specify the table only once.");
                }
                let t = *w
                    .get(i + 1)
                    .ok_or_else(|| fault(Class::Syntax, "Name the table after -t."))?;
                if t != "filter" {
                    return err(
                        Class::Unsupported,
                        "Only the IPv6 filter table is supported; no NAT/raw/mangle table ran.",
                    );
                }
                table_seen = true;
                w.drain(i..=i + 1);
                continue;
            }
            let count = match w[i] {
                "-P" | "--policy" => 3,
                "-A" | "--append" | "-I" | "--insert" | "-D" | "--delete" => 2,
                "-s" | "--source" | "-d" | "--destination" | "-i" | "--in-interface" | "-o"
                | "--out-interface" | "-p" | "--protocol" | "--sport" | "--source-port"
                | "--dport" | "--destination-port" | "--icmpv6-type" | "-j" | "--jump" => 2,
                _ => 1,
            };
            i += count;
        }
        let Some(op) = w.first().copied() else {
            return err(
                Class::Syntax,
                "Choose an operation such as -S, -A, -I, -D, -P or -F.",
            );
        };
        if matches!(op, "-S" | "--list-rules" | "-F" | "--flush") {
            if w.len() > 2 {
                return err(
                    Class::Unsupported,
                    "This listing/flush supports only an optional built-in chain.",
                );
            }
            let chain = w.get(1).map(|c| parse_chain(c)).transpose()?;
            if matches!(op, "-S" | "--list-rules") {
                return Ok(self.listing(chain));
            }
            for c in CHAINS {
                if chain.is_none_or(|v| c == v) {
                    self.chain_mut(c).rules.clear();
                }
            }
            return Ok(String::new());
        }
        if !matches!(
            op,
            "-P" | "--policy" | "-D" | "--delete" | "-A" | "--append" | "-I" | "--insert"
        ) {
            return err(
                Class::Unsupported,
                "Only -S, -A, -I, -D by number, -P and -F are modeled. Use -S to inspect this filter profile.",
            );
        }
        let chain =
            parse_chain(w.get(1).ok_or_else(|| {
                fault(Class::Syntax, "Name a built-in chain after the operation.")
            })?)?;
        match op {
            "-P" | "--policy" => {
                if w.len() != 3 {
                    return err(Class::Syntax, "Use -P CHAIN ACCEPT or -P CHAIN DROP.");
                }
                self.chain_mut(chain).policy = parse_verdict(w[2])?;
            }
            "-D" | "--delete" => {
                if w.len() != 3 {
                    return err(
                        Class::Unsupported,
                        "Delete by one-based rule number in this profile; rule-specification deletion is not yet supported.",
                    );
                }
                let n = index(w[2])?;
                let c = self.chain_mut(chain);
                if n > c.rules.len() {
                    return err(
                        Class::Syntax,
                        "That rule number does not exist; inspect -S first.",
                    );
                }
                c.rules.remove(n - 1);
            }
            "-A" | "--append" | "-I" | "--insert" => {
                let insert = matches!(op, "-I" | "--insert");
                let mut at = 2;
                let mut n = 1;
                if insert
                    && w.get(at)
                        .is_some_and(|s| s.bytes().all(|b| b.is_ascii_digit()))
                {
                    n = index(w[at])?;
                    at += 1;
                }
                let r = parse_rule(&w[at..], chain)?;
                let c = self.chain_mut(chain);
                if insert {
                    if n > c.rules.len() + 1 {
                        return err(
                            Class::Syntax,
                            "Insert at an existing rule number or one after the last rule.",
                        );
                    }
                    c.rules.insert(n - 1, r);
                } else {
                    c.rules.push(r);
                }
            }
            _ => {
                return err(
                    Class::Unsupported,
                    "Only -S, -A, -I, -D by number, -P and -F are modeled in this first filter profile.",
                );
            }
        }
        Ok(String::new())
    }
}
fn words(line: &str) -> Result<Vec<String>, Fault> {
    crate::linux_filter::words(line).map_err(|mut e| {
        e.message = e.message.replace("iptables", "ip6tables");
        e
    })
}
fn index(s: &str) -> Result<usize, Fault> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return err(Class::Syntax, "Use a positive one-based rule number.");
    }
    s.parse::<usize>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| fault(Class::Syntax, "Use a positive one-based rule number."))
}
fn port(s: &str) -> Result<u16, Fault> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return err(
            Class::Unsupported,
            "Use a numeric single port from 0 to 65535; ranges and service-name lookup are unmodeled.",
        );
    }
    s.parse::<u16>()
        .map_err(|_| fault(Class::Syntax, "Use a numeric port from 0 to 65535."))
}
fn parse_rule(w: &[&str], chain: Chain) -> Result<Rule, Fault> {
    let mut r = Rule {
        source: None,
        destination: None,
        ingress: None,
        egress: None,
        protocol: None,
        source_port: None,
        destination_port: None,
        icmp_type: None,
        icmp_code: None,
        target: Verdict::Drop,
    };
    let mut seen = BTreeSet::new();
    let mut i = 0;
    while i < w.len() {
        let k = match w[i] {
            "-s" | "--source" => "s",
            "-d" | "--destination" => "d",
            "-i" | "--in-interface" => "i",
            "-o" | "--out-interface" => "o",
            "-p" | "--protocol" => "p",
            "--sport" | "--source-port" => "sport",
            "--dport" | "--destination-port" => "dport",
            "--icmpv6-type" => "icmp6",
            "-j" | "--jump" => "j",
            _ => {
                return err(
                    Class::Unsupported,
                    "This match or operation is outside the numeric IPv6 filter profile; nothing changed.",
                );
            }
        };
        if !seen.insert(k) {
            return err(Class::Syntax, "Specify each match/target option only once.");
        }
        let v = *w
            .get(i + 1)
            .ok_or_else(|| fault(Class::Syntax, "Supply a value after each filter option."))?;
        match k {
            "s" => r.source = Some(Cidr::parse(v)?),
            "d" => r.destination = Some(Cidr::parse(v)?),
            "i" | "o" => {
                if !device(v) {
                    return err(
                        Class::Unsupported,
                        "Use an exact interface name of at most 15 ASCII bytes; no interface patterns.",
                    );
                }
                if k == "i" {
                    r.ingress = Some(v.into())
                } else {
                    r.egress = Some(v.into())
                }
            }
            "p" => {
                r.protocol = match v {
                    "all" => None,
                    "tcp" | "6" => Some(Protocol::Tcp),
                    "udp" | "17" => Some(Protocol::Udp),
                    "ipv6-icmp" | "icmpv6" | "58" => Some(Protocol::Icmp6),
                    _ => {
                        return err(
                            Class::Unsupported,
                            "Only tcp/6, udp/17, ipv6-icmp/icmpv6/58 and all are supported.",
                        );
                    }
                }
            }
            "sport" | "dport" => {
                if !matches!(r.protocol, Some(Protocol::Tcp | Protocol::Udp)) {
                    return err(
                        Class::Unsupported,
                        "In this profile, select -p tcp or -p udp before port matches; explicit match modules are unmodeled.",
                    );
                }
                if k == "sport" {
                    r.source_port = Some(port(v)?)
                } else {
                    r.destination_port = Some(port(v)?)
                }
            }
            "icmp6" => {
                if r.protocol != Some(Protocol::Icmp6) {
                    return err(
                        Class::Unsupported,
                        "Select -p ipv6-icmp or -p icmpv6 before --icmpv6-type.",
                    );
                }
                let (t, c) = v.split_once('/').map_or((v, None), |(a, b)| (a, Some(b)));
                let number = |value: &str| -> Result<u8, Fault> {
                    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                        return err(
                            Class::Unsupported,
                            "Use numeric ICMPv6 type[/code], each from 0 to 255; named types are outside this profile.",
                        );
                    }
                    value
                        .parse()
                        .map_err(|_| fault(Class::Syntax, "ICMPv6 type/code must be 0 to 255."))
                };
                r.icmp_type = Some(number(t)?);
                r.icmp_code = c.map(number).transpose()?;
            }
            "j" => r.target = parse_verdict(v)?,
            _ => unreachable!(),
        }
        i += 2;
    }
    if !seen.contains("j") {
        return err(
            Class::Unsupported,
            "Non-terminating counter-only rules are unmodeled; choose an explicit ACCEPT or DROP target.",
        );
    }
    if !r.valid(chain) {
        return err(
            Class::Syntax,
            "Use TCP/UDP for ports and IPv6 ICMP for type/code; ingress is invalid on OUTPUT, and egress is invalid on INPUT.",
        );
    }
    Ok(r)
}
