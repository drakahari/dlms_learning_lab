//! Bounded IPv4 NSS/DNS-search composition and a single-key getent subset.
//! No host resolver, filesystem, subprocess or network activity.
//! DNS search belongs to its NSS backend, not around the entire source sequence.
use crate::{client_names as names, resolver_search as search};
use std::net::Ipv4Addr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub name: String,
    pub address: Option<Ipv4Addr>,
    pub status: names::Status,
    pub steps: Vec<names::Step>,
    pub dns_attempts: Vec<search::Attempt>,
    pub answered_by: Option<names::Service>,
    pub canonical: Option<String>,
}
fn error(kind: names::Kind, message: impl Into<String>) -> names::Error {
    names::Error {
        kind,
        file: "Combined client lookup",
        line: None,
        message: message.into(),
    }
}

pub fn resolve(
    switch: &names::Switch,
    hosts: &names::Hosts,
    context: &names::Context,
    settings: &search::Settings,
    query: &str,
) -> Result<Resolution, names::Error> {
    context.validate()?;
    let base = query.strip_suffix('.').unwrap_or(query);
    if !names::name(base) {
        return Err(error(
            names::Kind::Invalid,
            "Use an ordinary supported name, optionally ending in one absolute dot.",
        ));
    }
    let mut result = Resolution {
        name: query.to_ascii_lowercase(),
        address: None,
        status: names::Status::NotFound,
        steps: Vec::new(),
        dns_attempts: Vec::new(),
        answered_by: None,
        canonical: None,
    };
    for source in switch.sources() {
        let (address, status) = match source.service {
            names::Service::Files => {
                if context.files_available {
                    // Deliberately retain the original query here, including final dot.
                    // Never look up each DNS search suffix in the local hosts file.
                    let address = hosts.names().get(&query.to_ascii_lowercase()).copied();
                    (
                        address,
                        if address.is_some() {
                            names::Status::Success
                        } else {
                            names::Status::NotFound
                        },
                    )
                } else {
                    (None, names::Status::Unavailable)
                }
            }
            names::Service::Dns => {
                // Do not let irrelevant DNS errors or overlong expansions prevent
                // an earlier successful local-files result. Validate at this backend.
                if !matches!(
                    context.dns.status,
                    names::Status::Success | names::Status::NotFound
                ) {
                    return Err(error(
                        names::Kind::Unsupported,
                        "Unavailable/retrying DNS search sequencing is outside this activity.",
                    ));
                }
                let found = settings.lookup(query, &context.dns)?;
                result.dns_attempts = found.attempts;
                (
                    found.address,
                    if found.address.is_some() {
                        names::Status::Success
                    } else {
                        names::Status::NotFound
                    },
                )
            }
        };
        let action = source.action(status);
        result.steps.push(names::Step {
            service: source.service,
            status,
            action,
            answer: address,
        });
        result.status = status;
        if action == names::Action::Return {
            result.address = address;
            if address.is_some() {
                result.answered_by = Some(source.service);
                result.canonical = match source.service {
                    names::Service::Files => hosts.canonical(query).map(str::to_owned),
                    names::Service::Dns => result.dns_attempts.last().map(|a| a.name.clone()),
                };
            }
            return Ok(result);
        }
    }
    Ok(result)
}

pub const MAX_COMMAND: usize = 512;
pub const MAX_OUTPUT: usize = 2048;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Query {
    Name(String),
    Numeric(Ipv4Addr),
}
pub fn parse_getent(raw: &str) -> Result<Query, names::Error> {
    if raw.len() > MAX_COMMAND || raw.chars().any(|c| c.is_control() && c != '\t') {
        return Err(error(
            names::Kind::Limit,
            "Keep one getent command within512bytes with no control syntax.",
        ));
    }
    let words: Vec<_> = raw.split_ascii_whitespace().take(5).collect();
    let ["getent", "ahostsv4", query] = words.as_slice() else {
        return Err(error(
            names::Kind::Unsupported,
            "Supported subset: getent ahostsv4 NAME. Other databases, flags, multiple keys and enumeration are outside this activity.",
        ));
    };
    if let Ok(address) = query.parse::<Ipv4Addr>() {
        return Ok(Query::Numeric(address));
    }
    let legacy_numeric = query.split('.').all(|part| {
        let hex = part.strip_prefix("0x").or_else(|| part.strip_prefix("0X"));
        hex.map_or_else(
            || !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()),
            |h| !h.is_empty() && h.bytes().all(|b| b.is_ascii_hexdigit()),
        )
    });
    if legacy_numeric {
        return Err(error(
            names::Kind::Unsupported,
            "Use four canonical decimal IPv4 octets. Legacy abbreviated, octal and hexadecimal numeric forms are not modeled.",
        ));
    }
    let base = query.strip_suffix('.').unwrap_or(query);
    if !names::name(base) {
        return Err(error(
            names::Kind::Invalid,
            "Use one supported host name or canonical IPv4 address, not a path, URL, IPv6 literal or shell expression.",
        ));
    }
    Ok(Query::Name((*query).into()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub stdout: String,
    pub status: u8,
    pub resolution: Option<Resolution>,
}
/// One IPv4 address expands to stream, datagram and raw socket-address rows.
/// This is a defined finite glibc-style subset, not a call to real getaddrinfo.
fn rows(address: Ipv4Addr, canonical: &str) -> String {
    format!("{address:<15} STREAM {canonical}\n{address:<15} DGRAM  \n{address:<15} RAW    \n")
}
pub fn getent(
    switch: &names::Switch,
    hosts: &names::Hosts,
    context: &names::Context,
    settings: &search::Settings,
    command: &str,
) -> Result<Output, names::Error> {
    let (stdout, status, resolution) = match parse_getent(command)? {
        Query::Numeric(address) => (rows(address, &address.to_string()), 0, None),
        Query::Name(query) => {
            let resolution = resolve(switch, hosts, context, settings, &query)?;
            let (stdout, status) = match resolution.address {
                Some(address) => (
                    rows(
                        address,
                        resolution
                            .canonical
                            .as_deref()
                            .expect("successful resolution has provenance"),
                    ),
                    0,
                ),
                None => (String::new(), 2),
            };
            (stdout, status, Some(resolution))
        }
    };
    debug_assert!(stdout.len() <= MAX_OUTPUT);
    Ok(Output {
        stdout,
        status,
        resolution,
    })
}
