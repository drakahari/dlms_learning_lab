//! Bounded ISC-style configuration parsing; pure data, not a DHCP daemon.
use crate::dhcpv4::Subnet;
use std::net::Ipv4Addr;
pub const MAX_TEXT: usize = 16_384;
pub const MAX_TOKENS: usize = 2048;
pub const MAX_SCOPES: usize = crate::dhcpv4::MAX_POOLS;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Syntax,
    Unsupported,
    Invalid,
    Limit,
}
impl ErrorKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Syntax => "Syntax error",
            Self::Unsupported => "Unsupported configuration",
            Self::Invalid => "Invalid configuration",
            Self::Limit => "Simulator limit",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: usize,
    pub column: usize,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 {
            return write!(f, "{}: {}", self.kind.label(), self.message);
        }
        write!(
            f,
            "{} at line {}, column {}: {}",
            self.kind.label(),
            self.line,
            self.column,
            self.message
        )
    }
}
#[derive(Clone, Copy, Debug)]
struct Token<'a> {
    text: &'a str,
    line: usize,
    column: usize,
}
impl Token<'_> {
    fn error(self, kind: ErrorKind, message: impl Into<String>) -> Error {
        Error {
            kind,
            line: self.line,
            column: self.column,
            message: message.into(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parameters {
    pub router: Option<Ipv4Addr>,
    pub dns: Option<Vec<Ipv4Addr>>,
    pub subnet_mask: Option<Ipv4Addr>,
    pub default_lease: Option<u32>,
    pub max_lease: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    pub subnet: Subnet,
    pub range: Option<(Ipv4Addr, Ipv4Addr)>,
    pub parameters: Parameters,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Configuration {
    pub defaults: Parameters,
    pub scopes: Vec<Scope>,
}
impl Scope {
    pub fn effective(&self, global: &Parameters) -> Parameters {
        Parameters {
            router: self.parameters.router.or(global.router),
            dns: self.parameters.dns.clone().or_else(|| global.dns.clone()),
            subnet_mask: self.parameters.subnet_mask.or(global.subnet_mask),
            default_lease: self.parameters.default_lease.or(global.default_lease),
            max_lease: self.parameters.max_lease.or(global.max_lease),
        }
    }
}
impl Parameters {
    /// No client-requested duration is modeled here. Do not call this lease negotiation.
    pub fn default_duration(&self) -> u32 {
        self.default_lease
            .unwrap_or(43_200)
            .min(self.max_lease.unwrap_or(86_400))
    }
}
fn lex(source: &str) -> Result<Vec<Token<'_>>, Error> {
    let first = Token {
        text: "",
        line: 1,
        column: 1,
    };
    if source.len() > MAX_TEXT {
        return Err(first.error(
            ErrorKind::Limit,
            "Configuration exceeds the16KiB simulator limit",
        ));
    }
    if let Some(offset) = source
        .bytes()
        .position(|b| (b < 32 && !matches!(b, b'\n' | b'\r' | b'\t')) || b == 127)
    {
        let prefix = &source[..offset];
        let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        return Err(Token {
            text: "",
            line,
            column,
        }
        .error(
            ErrorKind::Syntax,
            "NUL and other control characters are not configuration text",
        ));
    }
    let b = source.as_bytes();
    let (mut at, mut line, mut column) = (0, 1, 1);
    let mut out = vec![];
    while at < b.len() {
        match b[at] {
            b' ' | b'\t' | b'\r' => {
                at += 1;
                column += 1;
                continue;
            }
            b'\n' => {
                at += 1;
                line += 1;
                column = 1;
                continue;
            }
            b'#' => {
                while at < b.len() && b[at] != b'\n' {
                    if !b[at].is_ascii() {
                        return Err(Token {
                            text: "",
                            line,
                            column,
                        }
                        .error(
                            ErrorKind::Unsupported,
                            "Only ASCII configuration text is supported",
                        ));
                    }
                    at += 1;
                    column += 1;
                }
                continue;
            }
            _ => {}
        }
        let start = at;
        let pos = Token {
            text: "",
            line,
            column,
        };
        if b[at] == b'"' || b[at] == b'\'' {
            return Err(pos.error(
                ErrorKind::Unsupported,
                "Quoted values and file includes are outside this configuration subset",
            ));
        }
        if matches!(b[at], b';' | b'{' | b'}' | b',') {
            at += 1;
            column += 1;
        } else {
            while at < b.len()
                && !b[at].is_ascii_whitespace()
                && !matches!(b[at], b';' | b'{' | b'}' | b',' | b'#')
            {
                if !b[at].is_ascii_graphic() || matches!(b[at], b'"' | b'\'') {
                    return Err(Token {
                        text: "",
                        line,
                        column,
                    }
                    .error(
                        ErrorKind::Unsupported,
                        "Only unquoted ASCII values are supported",
                    ));
                }
                at += 1;
                column += 1;
                if at - start > 64 {
                    return Err(pos.error(ErrorKind::Limit, "A configuration token exceeds64bytes"));
                }
            }
            if at == start {
                return Err(pos.error(ErrorKind::Syntax, "Unexpected control character"));
            }
        }
        if out.len() == MAX_TOKENS {
            return Err(pos.error(ErrorKind::Limit, "Configuration has too many tokens"));
        }
        out.push(Token {
            text: &source[start..at],
            ..pos
        });
    }
    Ok(out)
}
struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    at: usize,
    eof: Token<'a>,
}
impl<'a> Parser<'a> {
    fn here(&self) -> Token<'a> {
        self.tokens.get(self.at).copied().unwrap_or(self.eof)
    }
    fn take(&mut self) -> Result<Token<'a>, Error> {
        let t = self.here();
        if self.at == self.tokens.len() {
            Err(t.error(ErrorKind::Syntax, "Unexpected end of configuration"))
        } else {
            self.at += 1;
            Ok(t)
        }
    }
    fn is(&self, text: &str) -> bool {
        self.tokens
            .get(self.at)
            .is_some_and(|t| t.text.eq_ignore_ascii_case(text))
    }
    fn expect(&mut self, text: &str) -> Result<(), Error> {
        let t = self.take()?;
        if t.text.eq_ignore_ascii_case(text) {
            Ok(())
        } else {
            Err(t.error(
                ErrorKind::Syntax,
                format!("Expected {text}; found {}", t.text),
            ))
        }
    }
    fn ip(&mut self) -> Result<(Ipv4Addr, Token<'a>), Error> {
        let t = self.take()?;
        if matches!(t.text, ";" | "{" | "}" | ",") {
            return Err(t.error(ErrorKind::Syntax, "Expected a dotted-decimal IPv4 address"));
        }
        let value = t.text.parse::<Ipv4Addr>().map_err(|_| {
            let numeric = t.text.bytes().all(|b| b.is_ascii_digit() || b == b'.');
            if numeric && !t.text.split('.').any(|p| p.len() > 1 && p.starts_with('0')) {
                t.error(ErrorKind::Invalid, "An IPv4 address needs four decimal octets from0 to255")
            } else {
                t.error(ErrorKind::Unsupported, "Use canonical dotted-decimal IPv4; hostname/IPv6/alternate notation is not supported")
            }
        })?;
        Ok((value, t))
    }
    fn seconds(&mut self) -> Result<u32, Error> {
        let t = self.take()?;
        if !t.text.bytes().all(|b| b.is_ascii_digit()) || t.text.is_empty() {
            return Err(t.error(ErrorKind::Syntax, "Lease time needs decimal seconds"));
        }
        t.text
            .parse()
            .map_err(|_| t.error(ErrorKind::Limit, "Lease time exceeds the32bit bound"))
    }
    fn parameter(&mut self, p: &mut Parameters) -> Result<(), Error> {
        let t = self.take()?;
        if matches!(t.text, ";" | "{" | "}" | ",") {
            return Err(t.error(
                ErrorKind::Syntax,
                "Expected a supported parameter or subnet declaration",
            ));
        }
        if t.text.eq_ignore_ascii_case("option") {
            let name = self.take()?;
            if name.text.eq_ignore_ascii_case("routers") {
                if p.router.is_some() {
                    return Err(name.error(
                        ErrorKind::Unsupported,
                        "Specify routers only once in each scope",
                    ));
                }
                p.router = Some(self.ip()?.0);
                if self.is(",") {
                    return Err(self.here().error(ErrorKind::Unsupported,"This model supports one router per scope; it must not silently discard a router list"));
                }
            } else if name.text.eq_ignore_ascii_case("domain-name-servers") {
                if p.dns.is_some() {
                    return Err(name.error(
                        ErrorKind::Unsupported,
                        "Specify DNS servers only once in each scope",
                    ));
                }
                let mut values = vec![self.ip()?.0];
                while self.is(",") {
                    self.take()?;
                    if values.len() == 4 {
                        return Err(self.here().error(
                            ErrorKind::Limit,
                            "This model supports at most four DNS addresses",
                        ));
                    }
                    values.push(self.ip()?.0);
                }
                p.dns = Some(values);
            } else if name.text.eq_ignore_ascii_case("subnet-mask") {
                if p.subnet_mask.is_some() {
                    return Err(name.error(
                        ErrorKind::Unsupported,
                        "Specify subnet-mask only once in each scope",
                    ));
                }
                p.subnet_mask = Some(self.ip()?.0);
            } else {
                return Err(name.error(
                    ErrorKind::Unsupported,
                    format!("Unsupported DHCP option: {}", name.text),
                ));
            }
        } else if t.text.eq_ignore_ascii_case("default-lease-time") {
            if p.default_lease.is_some() {
                return Err(t.error(
                    ErrorKind::Unsupported,
                    "Specify default-lease-time only once in each scope",
                ));
            }
            p.default_lease = Some(self.seconds()?);
        } else if t.text.eq_ignore_ascii_case("max-lease-time") {
            if p.max_lease.is_some() {
                return Err(t.error(
                    ErrorKind::Unsupported,
                    "Specify max-lease-time only once in each scope",
                ));
            }
            p.max_lease = Some(self.seconds()?);
        } else {
            return Err(t.error(
                ErrorKind::Unsupported,
                format!("Unknown or unsupported directive: {}", t.text),
            ));
        }
        self.expect(";")
    }
    fn scope(&mut self) -> Result<Scope, Error> {
        self.expect("subnet")?;
        let (network, npos) = self.ip()?;
        self.expect("netmask")?;
        let (mask, mpos) = self.ip()?;
        let value = u32::from(mask);
        let prefix = value.leading_ones() as u8;
        let expected = if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix)
        };
        if value != expected {
            return Err(mpos.error(
                ErrorKind::Invalid,
                "A subnet mask must have contiguous network bits",
            ));
        }
        if !(1..=30).contains(&prefix) {
            return Err(mpos.error(ErrorKind::Unsupported,"Conventional DHCP subnet exercises support/1 through/30; /31 and/32 are special cases, and/0 is outside this model"));
        }
        let subnet = Subnet { network, prefix };
        if !subnet.valid() {
            return Err(npos.error(
                ErrorKind::Invalid,
                "Use the network address in a subnet declaration; its host bits must be zero",
            ));
        }
        self.expect("{")?;
        let (mut parameters, mut range) = (Parameters::default(), None);
        while !self.is("}") {
            if self.is("range") {
                let pos = self.take()?;
                if range.is_some() {
                    return Err(pos.error(
                        ErrorKind::Unsupported,
                        "Multiple ranges in a subnet are not modeled",
                    ));
                }
                let (first, _) = self.ip()?;
                let last = if self.is(";") { first } else { self.ip()?.0 };
                self.expect(";")?;
                if !subnet.host(first) || !subnet.host(last) || u32::from(first) > u32::from(last) {
                    return Err(pos.error(
                        ErrorKind::Invalid,
                        "The range must run upward between usable addresses in its declared subnet",
                    ));
                }
                if u32::from(last) - u32::from(first) >= crate::dhcpv4::MAX_RANGE as u32 {
                    return Err(pos.error(
                        ErrorKind::Limit,
                        "The simulated dynamic range is limited to256 addresses",
                    ));
                }
                range = Some((first, last));
            } else {
                if range.is_some() {
                    return Err(self.here().error(
                        ErrorKind::Unsupported,
                        "Place subnet parameters before its range declaration in this subset",
                    ));
                }
                self.parameter(&mut parameters)?;
            }
        }
        self.expect("}")?;
        Ok(Scope {
            subnet,
            range,
            parameters,
        })
    }
}
pub fn parse(source: &str) -> Result<Configuration, Error> {
    let mut p = Parser {
        tokens: lex(source)?,
        at: 0,
        eof: Token {
            text: "",
            line: source.bytes().filter(|b| *b == b'\n').count() + 1,
            column: source.rsplit('\n').next().unwrap_or("").chars().count() + 1,
        },
    };
    let mut c = Configuration {
        defaults: Parameters::default(),
        scopes: vec![],
    };
    while p.at < p.tokens.len() {
        if p.is("subnet") {
            let pos = p.here();
            if c.scopes.len() == MAX_SCOPES {
                return Err(pos.error(ErrorKind::Limit, "At most eight subnets are modeled"));
            }
            let s = p.scope()?;
            if c.scopes
                .iter()
                .any(|q| q.subnet.contains(s.subnet.network) || s.subnet.contains(q.subnet.network))
            {
                return Err(pos.error(
                    ErrorKind::Unsupported,
                    "Duplicate or overlapping subnet declarations are outside this model",
                ));
            }
            c.scopes.push(s);
        } else {
            if !c.scopes.is_empty() {
                return Err(p.here().error(
                    ErrorKind::Unsupported,
                    "Place global parameters before subnet declarations in this subset",
                ));
            }
            p.parameter(&mut c.defaults)?;
        }
    }
    if c.scopes.is_empty() {
        return Err(p.here().error(
            ErrorKind::Invalid,
            "Declare at least one subnet; no served network is defined",
        ));
    }
    Ok(c)
}

fn compilation_error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        line: 0,
        column: 0,
        message: message.into().chars().take(384).collect(),
    }
}

/// Compile only representable settings into the existing logical lease model.
/// This does not start dhcpd, inspect interfaces, or establish ISC conformance.
pub fn compile(
    source: &str,
    topology: &crate::dhcpv4::Topology,
    relays: &std::collections::BTreeMap<String, crate::dhcpv4::Relay>,
) -> Result<crate::dhcpv4::Policy, Error> {
    use crate::dhcpv4 as core;
    topology.validate().map_err(|e| {
        compilation_error(
            if e.kind == core::ErrorKind::Limit {
                ErrorKind::Limit
            } else {
                ErrorKind::Invalid
            },
            format!("Supplied topology: {e}"),
        )
    })?;
    if relays.len() > core::MAX_SEGMENTS {
        return Err(compilation_error(
            ErrorKind::Limit,
            "Too many supplied relay policies",
        ));
    }
    let config = parse(source)?;
    let mut pools = std::collections::BTreeMap::new();
    for scope in &config.scopes {
        let effective = scope.effective(&config.defaults);
        let declared_mask = Ipv4Addr::from(u32::MAX << (32 - scope.subnet.prefix));
        if effective
            .subnet_mask
            .is_some_and(|mask| mask != declared_mask)
        {
            return Err(compilation_error(
                ErrorKind::Unsupported,
                format!(
                    "Subnet {}/{}: this lease model cannot advertise a mask different from its declared subnet. The parsed mask was retained, not silently replaced.",
                    scope.subnet.network, scope.subnet.prefix
                ),
            ));
        }
        let duration = effective.default_duration();
        if !(core::MIN_LEASE..=core::MAX_LEASE).contains(&duration) {
            return Err(compilation_error(
                ErrorKind::Unsupported,
                format!(
                    "Subnet {}/{}: effective duration {duration}s is outside the logical model's8..86400s range; this is a simulator boundary, not dhcpd syntax.",
                    scope.subnet.network, scope.subnet.prefix
                ),
            ));
        }
        if let Some((first, last)) = scope.range {
            let id =
                format!("net-{}-{}", scope.subnet.network, scope.subnet.prefix).replace('.', "-");
            pools.insert(
                id,
                core::Pool {
                    subnet: scope.subnet,
                    first,
                    last,
                    excluded: Default::default(),
                    reservations: Default::default(),
                    router: effective.router,
                    dns: effective.dns.unwrap_or_default(),
                    lease_seconds: duration,
                },
            );
        }
    }
    let policy = core::Policy {
        server_enabled: true,
        pools,
        relays: relays.clone(),
    };
    policy.validate(topology).map_err(|e| {
        compilation_error(ErrorKind::Invalid, format!("Supplied policy context: {e}"))
    })?;
    Ok(policy)
}
