//! Closed, Linux-style Nginx routing over supplied virtual requests. No I/O.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};

pub const MAX_CONFIG: usize = 8192;
pub const MAX_TOKENS: usize = 1024;
pub const MAX_LOCATIONS: usize = 32;
pub const MAX_PATH: usize = 128;
pub const MAX_TEXT: usize = 256;
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
fn fault(kind: FaultKind, line: Option<usize>, message: impl Into<String>) -> Fault {
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
#[derive(Clone, Debug, PartialEq, Eq)]
enum Kind {
    Word(String),
    Open,
    Close,
    Semi,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Token {
    kind: Kind,
    line: usize,
}
fn tokens(text: &str) -> Result<Vec<Token>> {
    if text.len() > MAX_CONFIG {
        return Err(fault(
            FaultKind::Limit,
            None,
            "Keep this server fragment within 8192 bytes.",
        ));
    }
    if !text.is_ascii()
        || text
            .bytes()
            .any(|b| b.is_ascii_control() && !matches!(b, b'\n' | b'\r' | b'\t'))
    {
        return Err(fault(
            FaultKind::Unsupported,
            None,
            "This configuration exercise supports printable ASCII, tabs and newlines only.",
        ));
    }
    let bytes = text.as_bytes();
    let mut at = 0;
    let mut line = 1;
    let mut out = Vec::new();
    while at < bytes.len() {
        match bytes[at] {
            b'\n' => {
                line += 1;
                at += 1;
                continue;
            }
            b' ' | b'\r' | b'\t' => {
                at += 1;
                continue;
            }
            b'#' => {
                while at < bytes.len() && bytes[at] != b'\n' {
                    at += 1;
                }
                continue;
            }
            _ => {}
        }
        let start_line = line;
        let kind = match bytes[at] {
            b'{' => {
                at += 1;
                Kind::Open
            }
            b'}' => {
                at += 1;
                Kind::Close
            }
            b';' => {
                at += 1;
                Kind::Semi
            }
            b'\'' | b'"' => {
                let quote = bytes[at];
                at += 1;
                let start = at;
                while at < bytes.len() && bytes[at] != quote {
                    if matches!(bytes[at], b'\n' | b'\r' | b'\\' | b'$') {
                        return Err(fault(
                            FaultKind::Unsupported,
                            Some(line),
                            "Quoted literals cannot contain newlines, escapes or variables in this subset.",
                        ));
                    }
                    at += 1;
                }
                if at == bytes.len() {
                    return Err(fault(
                        FaultKind::Syntax,
                        Some(line),
                        "Close the quoted literal before the end of the fragment.",
                    ));
                }
                let word = text[start..at].to_owned();
                at += 1;
                if at < bytes.len()
                    && !bytes[at].is_ascii_whitespace()
                    && !matches!(bytes[at], b';' | b'{' | b'}' | b'#')
                {
                    return Err(fault(
                        FaultKind::Unsupported,
                        Some(line),
                        "Keep a quoted value separate from adjacent tokens; concatenated literals are unsupported.",
                    ));
                }
                Kind::Word(word)
            }
            _ => {
                let start = at;
                while at < bytes.len()
                    && !bytes[at].is_ascii_whitespace()
                    && !matches!(bytes[at], b'{' | b'}' | b';' | b'#')
                {
                    if matches!(bytes[at], b'\'' | b'"' | b'\\' | b'$') {
                        return Err(fault(
                            FaultKind::Unsupported,
                            Some(line),
                            "Variables, escapes and quotes inside an unquoted token are unsupported.",
                        ));
                    }
                    at += 1;
                }
                Kind::Word(text[start..at].to_owned())
            }
        };
        if matches!(&kind,Kind::Word(w) if w.len()>MAX_TEXT) {
            return Err(fault(
                FaultKind::Limit,
                Some(start_line),
                "Keep each token within 256 bytes.",
            ));
        }
        out.push(Token {
            kind,
            line: start_line,
        });
        if out.len() > MAX_TOKENS {
            return Err(fault(
                FaultKind::Limit,
                Some(start_line),
                "The fragment exceeds 1024 tokens.",
            ));
        }
    }
    Ok(out)
}

/// Already normalized Linux exercise paths, not a URI normalizer.
pub fn path(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > MAX_PATH || !value.starts_with('/') {
        return Err(fault(
            FaultKind::Invalid,
            None,
            "Use an absolute path beginning with /, within 128 bytes.",
        ));
    }
    if !value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"/-._~".contains(&b))
        || value.contains("//")
        || value.split('/').any(|p| matches!(p, "." | ".."))
    {
        return Err(fault(
            FaultKind::Unsupported,
            None,
            "Use a normalized ASCII path: query strings, percent encoding, repeated slashes and dot segments are outside this exercise.",
        ));
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub address: Ipv4Addr,
    pub port: u16,
}
impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.address, self.port)
    }
}
impl Endpoint {
    pub fn validate(self) -> Result<()> {
        if self.port == 0 {
            Err(fault(
                FaultKind::Invalid,
                None,
                "A virtual endpoint needs a port from 1 to 65535.",
            ))
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proxy {
    pub endpoint: Endpoint,
    pub uri: Option<String>,
}
fn proxy(value: &str) -> Result<Proxy> {
    let rest=value.strip_prefix("http://").ok_or_else(||fault(FaultKind::Unsupported,None,"Use a literal http:// IPv4 endpoint; HTTPS, names, variables and sockets are unsupported."))?;
    let (authority, uri) = rest
        .find('/')
        .map_or((rest, None), |i| (&rest[..i], Some(rest[i..].to_owned())));
    if authority.contains('[') || authority.contains(']') {
        return Err(fault(
            FaultKind::Unsupported,
            None,
            "IPv6 proxy endpoints are outside this IPv4 routing subset.",
        ));
    }
    let (host, port) = match authority.split_once(':') {
        Some((host, port)) => (host, decimal_port(port)?),
        None => (authority, 80),
    };
    let address = host.parse::<Ipv4Addr>().map_err(|_| {
        fault(
            FaultKind::Unsupported,
            None,
            "Use a dotted-decimal IPv4 literal; DNS names and IPv6 endpoints are not modeled here.",
        )
    })?;
    if let Some(uri) = &uri {
        path(uri)?;
    }
    Ok(Proxy {
        endpoint: Endpoint { address, port },
        uri,
    })
}
fn decimal_port(value: &str) -> Result<u16> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(fault(
            FaultKind::Syntax,
            None,
            "Use one decimal TCP port from 1 to 65535.",
        ));
    }
    value
        .parse::<u16>()
        .ok()
        .filter(|p| *p != 0)
        .ok_or_else(|| fault(FaultKind::Invalid, None, "Use a TCP port from 1 to 65535."))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Proxy(Proxy),
    Return { status: u16, body: Option<String> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub exact: bool,
    pub path: String,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    listen: u16,
    server_name: Option<String>,
    locations: Vec<Location>,
}
struct Parser {
    tokens: Vec<Token>,
    at: usize,
}
impl Parser {
    fn line(&self) -> Option<usize> {
        self.tokens
            .get(self.at)
            .map(|t| t.line)
            .or_else(|| self.tokens.last().map(|t| t.line))
    }
    fn error(&self, message: &str) -> Fault {
        fault(FaultKind::Syntax, self.line(), message)
    }
    fn word(&mut self) -> Result<String> {
        match self.tokens.get(self.at).map(|t| &t.kind) {
            Some(Kind::Word(w)) => {
                let w = w.clone();
                self.at += 1;
                Ok(w)
            }
            _ => Err(self.error("Expected a directive name or value here.")),
        }
    }
    fn take(&mut self, kind: Kind, message: &str) -> Result<()> {
        if self.tokens.get(self.at).is_some_and(|t| t.kind == kind) {
            self.at += 1;
            Ok(())
        } else {
            Err(self.error(message))
        }
    }
    fn end(&self) -> bool {
        self.tokens
            .get(self.at)
            .is_some_and(|t| t.kind == Kind::Close)
    }
    fn args(&mut self) -> Result<Vec<String>> {
        let mut args = Vec::new();
        while matches!(
            self.tokens.get(self.at).map(|t| &t.kind),
            Some(Kind::Word(_))
        ) {
            args.push(self.word()?);
            if args.len() > 3 {
                return Err(
                    self.error("This directive has too many values for the supported subset.")
                );
            }
        }
        self.take(Kind::Semi, "End each directive with a semicolon (;).")?;
        Ok(args)
    }
    fn location(&mut self) -> Result<Location> {
        let first = self.word()?;
        let (exact, uri) = if matches!(first.as_str(), "=" | "^~") {
            (first == "=", self.word()?)
        } else {
            (false, first)
        };
        if uri.starts_with('@') || matches!(uri.as_str(), "~" | "~*") {
            return Err(fault(
                FaultKind::Unsupported,
                self.line(),
                "Use a simple prefix or = exact location; named and regular-expression locations are unsupported.",
            ));
        }
        path(&uri)?;
        self.take(Kind::Open, "Open this location with {.")?;
        let directive = self.word()?;
        let args = self.args()?;
        let action = match directive.as_str() {
            "proxy_pass" if args.len() == 1 => Action::Proxy(proxy(&args[0])?),
            "return" if (1..=2).contains(&args.len()) => {
                let status = args[0]
                    .parse::<u16>()
                    .map_err(|_| self.error("Use a numeric return status."))?;
                if !matches!(status, 200 | 403 | 404 | 503) {
                    return Err(fault(
                        FaultKind::Unsupported,
                        self.line(),
                        "The return subset supports200,403,404 and503; redirect URLs and other status behaviors are not modeled.",
                    ));
                }
                Action::Return {
                    status,
                    body: args.get(1).cloned(),
                }
            }
            "proxy_pass" | "return" => return Err(self.error(
                "Use one proxy_pass URL, or return STATUS with one optional quoted literal body.",
            )),
            _ => {
                return Err(fault(
                    FaultKind::Unsupported,
                    self.line(),
                    format!(
                        "{directive} is outside this routing subset. A location supports one proxy_pass or return action."
                    ),
                ));
            }
        };
        self.take(Kind::Close,"Close this location with a closing brace (}). Nested locations, extra directives and inherited actions are unsupported.")?;
        Ok(Location {
            exact,
            path: uri,
            action,
        })
    }
    fn config(&mut self) -> Result<Config> {
        if self.word()? != "server" {
            return Err(fault(
                FaultKind::Unsupported,
                self.line(),
                "Edit one server { ... } fragment for the http context; complete nginx.conf wrappers and other top-level directives are unsupported.",
            ));
        }
        self.take(Kind::Open, "Open the server block with {.")?;
        let mut listen = None;
        let mut server_name = None;
        let mut locations = Vec::new();
        let mut seen = BTreeSet::new();
        while !self.end() {
            let directive = self.word()?;
            match directive.as_str() {
                "listen" => {
                    let args = self.args()?;
                    if args.len() != 1 || listen.is_some() {
                        return Err(fault(
                            FaultKind::Unsupported,
                            self.line(),
                            "Use one decimal listen port only; addresses, multiple listeners, flags and TLS are outside this subset.",
                        ));
                    }
                    listen = Some(decimal_port(&args[0])?);
                }
                "server_name" => {
                    let args = self.args()?;
                    if args.len() != 1
                        || server_name.is_some()
                        || args[0].is_empty()
                        || !args[0]
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b))
                    {
                        return Err(fault(
                            FaultKind::Unsupported,
                            self.line(),
                            "Use one literal server_name only; wildcard, regex and multi-server selection are unsupported.",
                        ));
                    }
                    server_name = Some(args[0].clone());
                }
                "location" => {
                    let loc = self.location()?;
                    if !seen.insert((loc.exact, loc.path.clone())) {
                        return Err(fault(
                            FaultKind::Syntax,
                            self.line(),
                            "Each prefix or exact location may appear only once.",
                        ));
                    }
                    locations.push(loc);
                    if locations.len() > MAX_LOCATIONS {
                        return Err(fault(
                            FaultKind::Limit,
                            self.line(),
                            "Use at most 32 locations.",
                        ));
                    }
                }
                _ => {
                    return Err(fault(
                        FaultKind::Unsupported,
                        self.line(),
                        format!(
                            "{directive} is outside the server-fragment subset. Supported directives are listen, server_name and location."
                        ),
                    ));
                }
            }
        }
        self.take(Kind::Close, "Close the server block with }.")?;
        if self.at != self.tokens.len() {
            return Err(fault(
                FaultKind::Unsupported,
                self.line(),
                "Only one server fragment is supported, without additional blocks.",
            ));
        }
        if !locations.iter().any(|l| !l.exact && l.path == "/") {
            return Err(fault(
                FaultKind::Unsupported,
                None,
                "Include a location / fallback with an explicit action. Implicit filesystem serving is outside this exercise.",
            ));
        }
        // Avoid guessing how an inclusive prefix sibling interacts with the
        // automatic slash redirect. An explicit exact sibling is supported.
        for loc in &locations {
            if !loc.exact
                && loc.path != "/"
                && loc.path.ends_with('/')
                && matches!(loc.action, Action::Proxy(_))
            {
                let bare = &loc.path[..loc.path.len() - 1];
                if locations
                    .iter()
                    .any(|other| !other.exact && other.path == bare)
                {
                    return Err(fault(
                        FaultKind::Unsupported,
                        None,
                        "A proxied slash location and its bare prefix sibling are outside this subset. Use an = exact bare-path override if needed.",
                    ));
                }
            }
        }
        let listen=listen.ok_or_else(||fault(FaultKind::Unsupported,None,"Set an explicit listen port in this exercise; privilege-dependent real Nginx defaults are not modeled."))?;
        Ok(Config {
            listen,
            server_name,
            locations,
        })
    }
}
pub fn parse(text: &str) -> Result<Config> {
    Parser {
        tokens: tokens(text)?,
        at: 0,
    }
    .config()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Route {
    NotListening,
    Return {
        location: String,
        status: u16,
        body: Option<String>,
    },
    Redirect {
        location: String,
        path: String,
    },
    Proxy {
        location: String,
        endpoint: Endpoint,
        path: String,
    },
}
impl Config {
    pub fn listen(&self) -> u16 {
        self.listen
    }
    pub fn server_name(&self) -> Option<&str> {
        self.server_name.as_deref()
    }
    /// One server is the default for its supplied port. Host-header matching is
    /// deliberately not represented; server_name is only its optional label.
    pub fn route(&self, port: u16, uri: &str) -> Result<Route> {
        path(uri)?;
        if port == 0 {
            return Err(fault(
                FaultKind::Invalid,
                None,
                "The virtual request needs a nonzero destination port.",
            ));
        }
        if port != self.listen {
            return Ok(Route::NotListening);
        }
        let exact = self.locations.iter().find(|l| l.exact && l.path == uri);
        if exact.is_none()
            && let Some(loc) = self.locations.iter().find(|l| {
                !l.exact
                    && l.path != "/"
                    && l.path.ends_with('/')
                    && l.path.strip_suffix('/') == Some(uri)
                    && matches!(l.action, Action::Proxy(_))
            })
        {
            return Ok(Route::Redirect {
                location: loc.path.clone(),
                path: loc.path.clone(),
            });
        }
        let loc = exact
            .or_else(|| {
                self.locations
                    .iter()
                    .filter(|l| !l.exact && uri.starts_with(&l.path))
                    .max_by_key(|l| l.path.len())
            })
            .ok_or_else(|| {
                fault(
                    FaultKind::Invalid,
                    None,
                    "The parsed configuration lost its required fallback location.",
                )
            })?;
        match &loc.action {
            Action::Return { status, body } => Ok(Route::Return {
                location: loc.path.clone(),
                status: *status,
                body: body.clone(),
            }),
            Action::Proxy(p) => {
                let target = match &p.uri {
                    None => uri.to_owned(),
                    Some(base) => format!("{base}{}", &uri[loc.path.len()..]),
                };
                // All pieces were bounded/validated; preserve the literal slash
                // concatenation. Do not silently normalize the upstream target.
                Ok(Route::Proxy {
                    location: loc.path.clone(),
                    endpoint: p.endpoint,
                    path: target,
                })
            }
        }
    }
}

/// Routing requirements exclude the explanatory name of the chosen location.
/// An absent required return body leaves default error-page bytes unconstrained.
/// Supported alternatives may use different location partitions.
fn same_route(a: &Route, b: &Route) -> bool {
    match (a, b) {
        (Route::NotListening, Route::NotListening) => true,
        (
            Route::Return {
                status: a,
                body: ab,
                ..
            },
            Route::Return {
                status: b,
                body: bb,
                ..
            },
        ) => {
            a == b
                && bb
                    .as_ref()
                    .is_none_or(|required| ab.as_ref() == Some(required))
        }
        (Route::Redirect { path: a, .. }, Route::Redirect { path: b, .. }) => a == b,
        (
            Route::Proxy {
                endpoint: a,
                path: ap,
                ..
            },
            Route::Proxy {
                endpoint: b,
                path: bp,
                ..
            },
        ) => a == b && ap == bp,
        _ => false,
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Difference {
    pub path: String,
    pub expected: Route,
    pub actual: Route,
}
/// A finite partition of the closed normalized path language. Every location
/// and slash-redirect boundary from BOTH configurations participates. At each
/// trie node, check the node and one absent valid next-character branch. Inside
/// that branch there is no later location boundary. A proxy's remainder is only
/// copied unchanged, so equality at its representative establishes equality for
/// that whole branch; return/redirect/not-listening are constant there. An
/// omitted required return body is unconstrained, not an assertion of empty bytes.
/// This is not a proof for unsupported regex, normalization or rewrite features.
pub fn difference(actual: &Config, expected: &Config, port: u16) -> Result<Option<Difference>> {
    let mut nodes = BTreeSet::from(["/".to_owned()]);
    for loc in actual.locations.iter().chain(&expected.locations) {
        for n in 1..=loc.path.len() {
            nodes.insert(loc.path[..n].to_owned());
        }
    }
    let mut probes = nodes
        .iter()
        .filter(|s| path(s).is_ok())
        .cloned()
        .collect::<BTreeSet<_>>();
    for node in &nodes {
        if node.len() >= MAX_PATH {
            continue;
        }
        for byte in b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~/" {
            let branch = format!("{node}{}", char::from(*byte));
            if !nodes.contains(&branch) && path(&branch).is_ok() {
                probes.insert(branch);
                break;
            }
        }
    }
    for uri in probes {
        let a = actual.route(port, &uri)?;
        let b = expected.route(port, &uri)?;
        if !same_route(&a, &b) {
            return Ok(Some(Difference {
                path: uri,
                expected: b,
                actual: a,
            }));
        }
    }
    Ok(None)
}

fn draft_text(text: &str) -> Result<()> {
    if text.len() > 32768
        || text.chars().count() > 8192
        || text
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        Err(fault(
            FaultKind::Limit,
            None,
            "Keep the virtual configuration within 8192 displayed characters and omit nonprinting controls.",
        ))
    } else {
        Ok(())
    }
}
/// Serialized raw virtual files only. No parsed tree or result can be imported
/// as authority. Applying always reparses; a failed apply leaves active data intact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    saved: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    applied: String,
}
impl World {
    pub fn new(text: &str) -> Result<Self> {
        parse(text)?;
        Ok(Self {
            saved: text.into(),
            applied: text.into(),
        })
    }
    pub fn validate(&self) -> Result<()> {
        draft_text(&self.saved)?;
        parse(&self.applied)?;
        Ok(())
    }
    pub fn save(&mut self, text: &str) -> Result<bool> {
        self.validate()?;
        draft_text(text)?;
        let changed = self.saved != text;
        if changed {
            self.saved = text.into();
        }
        Ok(changed)
    }
    pub fn validate_saved(&self) -> Result<()> {
        self.validate()?;
        parse(&self.saved)?;
        Ok(())
    }
    pub fn apply_saved(&mut self) -> Result<bool> {
        self.validate_saved()?;
        let changed = self.applied != self.saved;
        if changed {
            self.applied = self.saved.clone();
        }
        Ok(changed)
    }
    pub fn saved(&self) -> &str {
        &self.saved
    }
    pub fn applied(&self) -> &str {
        &self.applied
    }
    pub fn effective_route(&self, port: u16, path: &str) -> Result<Route> {
        self.validate()?;
        parse(&self.applied)?.route(port, path)
    }
}

#[cfg(test)]
mod tests;
