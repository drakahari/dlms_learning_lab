//! Literal, bounded Postfix configuration and relay decisions. No host I/O.
use std::collections::{BTreeMap, BTreeSet};
use std::net::Ipv4Addr;

pub const MAX_TEXT: usize = 8192;
pub const MAX_LINES: usize = 96;
pub const MAX_LIST: usize = 16;
pub const MAX_NOTICE: usize = 512;
pub const PARAMETERS: [&str; 7] = [
    "myhostname",
    "mydestination",
    "mynetworks",
    "relay_domains",
    "relayhost",
    "smtpd_relay_restrictions",
    "parent_domain_matches_subdomains",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub kind: ErrorKind,
    pub message: String,
}
impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Fault {}
fn fail<T>(kind: ErrorKind, message: impl Into<String>) -> Result<T, Fault> {
    let mut message = message.into();
    if message.len() > MAX_NOTICE {
        let mut end = MAX_NOTICE - 3;
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
        message.push_str("...");
    }
    Err(Fault { kind, message })
}
fn field<T>(key: &str, result: Result<T, Fault>) -> Result<T, Fault> {
    result.or_else(|e| fail(e.kind, format!("{key}: {}", e.message)))
}
pub fn domain(text: &str) -> Result<String, Fault> {
    let name = text.strip_suffix('.').unwrap_or(text);
    if !name.is_ascii()
        || name.contains(['$', '"', '\'', '/', '*', '[', ']'])
        || name.starts_with('.')
    {
        return fail(
            ErrorKind::Unsupported,
            "Use a literal ASCII DNS name; variables, quotes, maps and patterns are outside this subset.",
        );
    }
    if name.is_empty()
        || name.len() > 253
        || name.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return fail(
            ErrorKind::Invalid,
            "Use a DNS name with nonempty labels, such as mail.example.test.",
        );
    }
    Ok(name.to_ascii_lowercase())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ipv4Prefix {
    network: Ipv4Addr,
    prefix: u8,
}
impl Ipv4Prefix {
    pub fn parse(value: &str) -> Result<Self, Fault> {
        if value.contains(':') {
            return fail(
                ErrorKind::Unsupported,
                "This mail unit supports explicit IPv4 trust boundaries; IPv6 networks are not modeled yet.",
            );
        }
        if value
            .split_once('/')
            .is_some_and(|(_, prefix)| prefix.contains('.'))
        {
            return fail(
                ErrorKind::Unsupported,
                "Dotted netmasks are outside this trust-list subset; use an explicit /0 through /32 prefix.",
            );
        }
        let (address, prefix) = match value.split_once('/') {
            Some((address, prefix)) => (
                address,
                prefix
                    .parse::<u8>()
                    .ok()
                    .filter(|n| *n <= 32)
                    .ok_or_else(|| Fault {
                        kind: ErrorKind::Invalid,
                        message: "Use an IPv4 prefix from /0 through /32.".into(),
                    })?,
            ),
            None => (value, 32),
        };
        let address = address.parse::<Ipv4Addr>().map_err(|_| Fault {
            kind: ErrorKind::Invalid,
            message: "Use a dotted-decimal IPv4 address or an explicit IPv4 network/prefix.".into(),
        })?;
        let mask = if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix)
        };
        let network = Ipv4Addr::from(u32::from(address) & mask);
        if network != address {
            return fail(
                ErrorKind::Unsupported,
                format!(
                    "Use the explicit network boundary {network}/{prefix}; host bits in a trust-network entry are outside this subset."
                ),
            );
        }
        Ok(Self { network, prefix })
    }
    pub fn contains(&self, address: Ipv4Addr) -> bool {
        let mask = if self.prefix == 0 {
            0
        } else {
            u32::MAX << (32 - self.prefix)
        };
        u32::from(address) & mask == u32::from(self.network)
    }
    pub fn network(&self) -> Ipv4Addr {
        self.network
    }
    pub fn prefix(&self) -> u8 {
        self.prefix
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelayTarget {
    pub host: String,
    pub port: u16,
}
impl RelayTarget {
    pub fn parse(value: &str) -> Result<Self, Fault> {
        let Some(tail) = value.strip_prefix('[') else {
            return fail(
                ErrorKind::Unsupported,
                "Use a supplied bracketed relay host. Unbracketed MX selection is outside this exercise.",
            );
        };
        let Some((host, rest)) = tail.split_once(']') else {
            return fail(
                ErrorKind::Invalid,
                "Close the relay host's bracket, for example [relay.example.test].",
            );
        };
        if host.contains(':') {
            return fail(
                ErrorKind::Unsupported,
                "IPv6 relay endpoints are not modeled in this subset.",
            );
        }
        let host = match host.parse::<Ipv4Addr>() {
            Ok(ip) => ip.to_string(),
            Err(_) => domain(host)?,
        };
        let port = if rest.is_empty() {
            25
        } else {
            if rest.strip_prefix(':').is_some_and(|port| {
                !port.is_empty()
                    && port.bytes().all(|b| b.is_ascii_alphanumeric())
                    && port.bytes().any(|b| b.is_ascii_alphabetic())
            }) {
                return fail(
                    ErrorKind::Unsupported,
                    "Named relay services are outside this subset; use the supplied numeric port.",
                );
            }
            rest.strip_prefix(':')
                .and_then(|s| s.parse::<u16>().ok())
                .filter(|n| *n > 0)
                .ok_or_else(|| Fault {
                    kind: ErrorKind::Invalid,
                    message: "Use a numeric relay port from 1 through 65535 after the bracket."
                        .into(),
                })?
        };
        Ok(Self { host, port })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Restriction {
    PermitMyNetworks,
    RejectUnauthorized,
    DeferUnauthorized,
    Reject,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelayDecision {
    Permit,
    Reject,
    Defer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub hostname: String,
    pub destinations: BTreeSet<String>,
    pub networks: Vec<Ipv4Prefix>,
    pub relay_domains: BTreeSet<String>,
    pub relay: Option<RelayTarget>,
    pub restrictions: Vec<Restriction>,
    /// Last explicit logical values, not rewritten/expanded postconf output.
    pub parameters: BTreeMap<String, String>,
    pub warnings: Vec<String>,
}
fn tokens(value: &str) -> Result<Vec<&str>, Fault> {
    let values: Vec<_> = value
        .split(|c: char| c == ',' || c.is_ascii_whitespace())
        .filter(|s| !s.is_empty())
        .collect();
    if values.len() > MAX_LIST {
        return fail(
            ErrorKind::Limit,
            "Keep each represented list within 16 entries.",
        );
    }
    Ok(values)
}
fn domains(value: &str) -> Result<BTreeSet<String>, Fault> {
    tokens(value)?
        .into_iter()
        .map(|literal| {
            domain(literal)?;
            // Postfix matches configured destination patterns literally after casefold.
            // Recipient-domain rewriting strips one final dot; it does not rewrite
            // the patterns in mydestination or relay_domains.
            Ok(literal.to_ascii_lowercase())
        })
        .collect()
}
/// Literal main.cf text, before parameter-specific semantic validation.
/// `postconf -n` can display explicit but semantically mistaken values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub parameters: BTreeMap<String, String>,
    pub warnings: Vec<String>,
}
impl Document {
    pub fn parse(text: &str) -> Result<Self, Fault> {
        if text.len() > MAX_TEXT || text.lines().count() > MAX_LINES {
            return fail(
                ErrorKind::Limit,
                "Keep the simulated main.cf within 8 KiB and 96 lines.",
            );
        }
        if text.contains('\0')
            || text
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        {
            return fail(
                ErrorKind::Invalid,
                "Remove control characters from this configuration.",
            );
        }
        let mut logical: Vec<(usize, String)> = vec![];
        for (n, line) in text.lines().enumerate() {
            let trimmed = line.trim_ascii();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if line.starts_with([' ', '\t']) {
                let Some((_, previous)) = logical.last_mut() else {
                    return fail(
                        ErrorKind::Invalid,
                        format!("Line {}: a continuation needs a preceding setting.", n + 1),
                    );
                };
                previous.push(' ');
                previous.push_str(trimmed);
            } else {
                logical.push((n + 1, trimmed.into()));
            }
        }
        let mut parameters = BTreeMap::new();
        let mut warnings = vec![];
        for (line, value) in logical {
            let Some((key, value)) = value.split_once('=') else {
                return fail(
                    ErrorKind::Invalid,
                    format!("Line {line}: write setting = value."),
                );
            };
            let key = key.trim_ascii();
            let value = value.trim_ascii();
            if !PARAMETERS.contains(&key) {
                return fail(
                    ErrorKind::Unsupported,
                    format!(
                        "Line {line}: {key} is not represented by this exercise. It is not a claim that Postfix rejects that setting."
                    ),
                );
            }
            if value.contains(['$', '#', '"', '\'']) {
                return fail(
                    ErrorKind::Unsupported,
                    format!(
                        "Line {line}: use literal values and put comments on their own lines. Variables, quoted values and inline # characters are outside this subset."
                    ),
                );
            }
            if parameters
                .insert(key.to_owned(), value.to_owned())
                .is_some()
            {
                warnings.push(format!("Line {line}: {key} overrides its earlier value."));
            }
        }
        Ok(Self {
            parameters,
            warnings,
        })
    }
}
impl Config {
    pub fn parse(text: &str) -> Result<Self, Fault> {
        let Document {
            parameters,
            warnings,
        } = Document::parse(text)?;
        for key in PARAMETERS {
            if !parameters.contains_key(key) {
                return fail(
                    ErrorKind::Unsupported,
                    format!(
                        "Keep the supplied explicit {key} setting. Real Postfix has defaults; this exercise does not invent them for missing settings."
                    ),
                );
            }
        }
        if !parameters["parent_domain_matches_subdomains"].is_empty() {
            return fail(
                ErrorKind::Unsupported,
                "This unit supplies empty parent_domain_matches_subdomains: domain names match exactly. Implicit subdomain policy is outside its subset.",
            );
        }
        let mut restrictions = vec![];
        for token in tokens(&parameters["smtpd_relay_restrictions"])? {
            restrictions.push(match token {
                "permit_mynetworks" => Restriction::PermitMyNetworks,
                "reject_unauth_destination" => Restriction::RejectUnauthorized,
                "defer_unauth_destination" => Restriction::DeferUnauthorized,
                "reject" => Restriction::Reject,
                _ => return fail(ErrorKind::Unsupported, format!("Relay restriction {token} is outside the supported literal policy subset.")),
            });
        }
        if !restrictions
            .iter()
            .any(|r| *r != Restriction::PermitMyNetworks)
        {
            return fail(
                ErrorKind::Invalid,
                "A relay policy needs a supported destination guard or reject restriction; trusted-network permission alone is insufficient.",
            );
        }
        Ok(Self {
            hostname: field("myhostname", domain(&parameters["myhostname"]))?,
            destinations: field("mydestination", domains(&parameters["mydestination"]))?,
            networks: field(
                "mynetworks",
                tokens(&parameters["mynetworks"])?
                    .into_iter()
                    .map(Ipv4Prefix::parse)
                    .collect::<Result<_, _>>(),
            )?,
            relay_domains: field("relay_domains", domains(&parameters["relay_domains"]))?,
            relay: if parameters["relayhost"].is_empty() {
                None
            } else {
                Some(field(
                    "relayhost",
                    RelayTarget::parse(&parameters["relayhost"]),
                )?)
            },
            restrictions,
            parameters,
            warnings,
        })
    }
    pub fn relay_decision(
        &self,
        client: Ipv4Addr,
        destination: &str,
    ) -> Result<RelayDecision, Fault> {
        let destination = domain(destination)?;
        let authorized =
            self.destinations.contains(&destination) || self.relay_domains.contains(&destination);
        Ok(self.policy_decision(self.networks.iter().any(|n| n.contains(client)), authorized))
    }
    /// Symbolic policy behavior, independent of which network/domain class supplied it.
    pub fn policy_decision(&self, trusted: bool, authorized: bool) -> RelayDecision {
        for rule in &self.restrictions {
            match rule {
                Restriction::PermitMyNetworks if trusted => {
                    return RelayDecision::Permit;
                }
                Restriction::RejectUnauthorized if !authorized => return RelayDecision::Reject,
                Restriction::DeferUnauthorized if !authorized => return RelayDecision::Defer,
                Restriction::Reject => return RelayDecision::Reject,
                _ => {} // Authorized-destination checks continue; they are not a blanket permit.
            }
        }
        RelayDecision::Permit
    }
}
