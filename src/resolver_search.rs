//! Bounded DNS search plans over supplied positive/missing IPv4 replies only.
//! No host resolver, files, network or process use.
use crate::client_names::{self as names, Dns, Error, Kind, Status};
use std::net::Ipv4Addr;
pub const MAX_TEXT: usize = 16_384;
pub const MAX_SUFFIXES: usize = 3;
fn error(kind: Kind, line: Option<usize>, message: impl Into<String>) -> Error {
    Error {
        kind,
        file: "resolv.conf search excerpt",
        line,
        message: message.into(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    search: Vec<String>,
    ndots: u8,
}
fn suffix(raw: &str) -> Option<String> {
    if raw == "." {
        Some(".".into())
    } else {
        let raw = raw.strip_suffix('.').unwrap_or(raw);
        names::name(raw).then(|| raw.to_ascii_lowercase())
    }
}
impl Settings {
    pub fn search(&self) -> &[String] {
        &self.search
    }
    pub fn ndots(&self) -> u8 {
        self.ndots
    }
    pub fn parse(raw: &str, hostname: &str) -> Result<Self, Error> {
        if raw.len() > MAX_TEXT || raw.lines().count() > 128 {
            return Err(error(Kind::Limit, None, "Use at most16KiB and128 lines"));
        }
        if raw
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        {
            return Err(error(Kind::Invalid, None, "Unsupported control character"));
        }
        if !names::name(hostname) {
            return Err(error(
                Kind::Invalid,
                None,
                "The sample hostname must be an ordinary supported name",
            ));
        }
        let default = hostname
            .split_once('.')
            .map(|(_, d)| d.to_ascii_lowercase())
            .unwrap_or_else(|| ".".into());
        let mut settings = Self {
            search: vec![default],
            ndots: 1,
        };
        for (i, row) in raw.lines().enumerate() {
            let line = Some(i + 1);
            let row = row.trim();
            if row.is_empty() || row.starts_with('#') || row.starts_with(';') {
                continue;
            }
            let fields = row.split_ascii_whitespace().take(6).collect::<Vec<_>>();
            if fields.len() > 5 {
                return Err(error(Kind::Limit, line, "Too many settings on a line"));
            }
            if fields.iter().any(|s| s.contains(['#', ';'])) {
                return Err(error(
                    Kind::Unsupported,
                    line,
                    "Use whole-line comments in this excerpt; inline comment syntax is outside this activity",
                ));
            }
            match fields[0] {
                "search" | "domain" => {
                    if fields.len() < 2 {
                        return Err(error(
                            Kind::Invalid,
                            line,
                            "Give search a suffix list, or domain one suffix; use . for the root",
                        ));
                    }
                    if fields[0] == "domain" && fields.len() != 2 {
                        return Err(error(
                            Kind::Unsupported,
                            line,
                            "domain takes one suffix in this subset; prefer search for a list",
                        ));
                    }
                    if fields.len() - 1 > MAX_SUFFIXES {
                        return Err(error(
                            Kind::Limit,
                            line,
                            "At most three search suffixes are supported here",
                        ));
                    }
                    let mut domains = vec![];
                    for word in &fields[1..] {
                        domains.push(suffix(word).ok_or_else(||error(Kind::Invalid,line,"Use an ordinary domain suffix, with an optional final dot, or . for the root"))?);
                    }
                    settings.search = domains;
                }
                "options" => {
                    if fields.len() < 2 {
                        return Err(error(Kind::Invalid, line, "Use options ndots:N"));
                    }
                    for word in &fields[1..] {
                        let Some(value) = word.strip_prefix("ndots:") else {
                            return Err(error(
                                Kind::Unsupported,
                                line,
                                "This excerpt supports only the ndots option, not other resolver options",
                            ));
                        };
                        if value.is_empty() || !value.bytes().all(|c| c.is_ascii_digit()) {
                            return Err(error(
                                Kind::Invalid,
                                line,
                                "ndots needs a decimal whole number",
                            ));
                        }
                        let number = value.parse::<u8>().map_err(|_| {
                            error(Kind::Limit, line, "Supported ndots range is0–15")
                        })?;
                        if number > 15 {
                            return Err(error(
                                Kind::Unsupported,
                                line,
                                "Use ndots0–15 in this activity; real glibc caps larger values at15",
                            ));
                        }
                        settings.ndots = number;
                    }
                }
                _ => {
                    return Err(error(
                        Kind::Unsupported,
                        line,
                        "Edit only search, domain and options ndots here; nameserver routing and other directives are outside this excerpt",
                    ));
                }
            }
        }
        Ok(settings)
    }
    pub fn plan(&self, query: &str) -> Result<Vec<String>, Error> {
        let absolute = query.ends_with('.');
        let base = if absolute {
            &query[..query.len() - 1]
        } else {
            query
        };
        if !names::name(base) {
            return Err(error(
                Kind::Invalid,
                None,
                "Use an ordinary DNS name, optionally ending in one absolute dot; numeric addresses, paths and shell syntax are not names here",
            ));
        }
        let base = base.to_ascii_lowercase();
        if absolute {
            return Ok(vec![base]);
        }
        let dots = base.bytes().filter(|c| *c == b'.').count();
        let first = dots >= usize::from(self.ndots);
        let mut plan = vec![];
        if first {
            plan.push(base.clone());
        }
        let mut root = false;
        for domain in &self.search {
            let name = if domain == "." {
                root = true;
                base.clone()
            } else {
                format!("{base}.{domain}")
            };
            if !names::name(&name) {
                return Err(error(
                    Kind::Limit,
                    None,
                    "A search-expanded name exceeds the supported253-byte label/name bounds; no query was run",
                ));
            }
            plan.push(name);
        }
        if !first && !root {
            plan.push(base);
        }
        debug_assert!(plan.len() <= 4);
        Ok(plan)
    }
    pub fn lookup(&self, query: &str, dns: &Dns) -> Result<Lookup, Error> {
        if dns.records.len() > 64 {
            return Err(error(Kind::Limit, None, "At most64 supplied A records"));
        }
        if dns
            .records
            .keys()
            .any(|name| !names::name(name) || name != &name.to_ascii_lowercase())
        {
            return Err(error(
                Kind::Invalid,
                None,
                "Supplied DNS record names must be lowercase supported names",
            ));
        }
        if dns.status == Status::NotFound && !dns.records.is_empty() {
            return Err(error(
                Kind::Invalid,
                None,
                "A missing-name-only sample cannot also contain successful answers",
            ));
        }
        if !matches!(dns.status, Status::Success | Status::NotFound) {
            return Err(error(
                Kind::Unsupported,
                None,
                "Search sequencing for unavailable/retrying DNS is outside this positive/missing-reply activity",
            ));
        }
        let plan = self.plan(query)?;
        let mut attempts = vec![];
        for candidate in plan {
            let address = dns.records.get(&candidate).copied();
            attempts.push(Attempt {
                name: candidate,
                address,
            });
            if address.is_some() {
                return Ok(Lookup { address, attempts });
            }
        }
        Ok(Lookup {
            address: None,
            attempts,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub name: String,
    pub address: Option<Ipv4Addr>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lookup {
    pub address: Option<Ipv4Addr>,
    pub attempts: Vec<Attempt>,
}
