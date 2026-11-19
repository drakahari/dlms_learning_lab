//! Opt-in direct A-query observation against supplied host facts only.
//! No sockets, host resolver, files, clocks, retries or command execution.
use crate::linux_host;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

pub const HELP: &str = "This lesson supports dig @SERVER NAME. A +short (also -t A, and +short before the query). SERVER must be a supplied numeric IPv4 resolver and NAME. an absolute supplied .lab.test name. A asks for an IPv4 address; +short displays just that answer. Explicit @SERVER bypasses the configured nameserver list and application/NSS lookup. Equivalent typed lookup: host -t A NAME. SERVER (also host -4 -t A NAME. SERVER), with a supplied numeric server and the same absolute positive-A names. Default/multi-type/reverse host lookups are unsupported. Only known positive A records are supplied: an unknown name/type is unsupported, not an invented NXDOMAIN or empty answer. No host dig, packets, timeouts, caches, recursion, TTL/header output, files, pipes or other options.";
pub const MAX_COMMAND: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Ipv4Addr,2>")]
    pub servers: Vec<Ipv4Addr>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub initial: Plan,
    pub example: Plan,
}
impl Plan {
    pub fn validate(&self, world: &linux_host::State) -> Result<(), String> {
        world.validate()?;
        if self.servers.is_empty() || self.servers.len() > 2 {
            return Err("Direct DNS needs one or two supplied IPv4 resolvers.".into());
        }
        for (i, server) in self.servers.iter().enumerate() {
            if self.servers[..i].contains(server)
                || !world
                    .profile
                    .resolvers
                    .iter()
                    .any(|r| r.address == *server && !r.records.is_empty())
            {
                return Err("Direct DNS servers must be distinct supplied resolvers with explicit positive A facts.".into());
            }
        }
        Ok(())
    }
    pub fn observe(&self, world: &linux_host::State, input: &str) -> Result<Observation, String> {
        self.validate(world)?;
        let query = Query::parse(input)?;
        if !self.servers.contains(&query.server) {
            return Err("Unsupported server: choose one of this lesson's supplied numeric IPv4 resolvers. No network query was attempted.".into());
        }
        let resolver = world
            .profile
            .resolvers
            .iter()
            .find(|r| r.address == query.server)
            .ok_or("Missing supplied DNS facts")?;
        let answer = resolver.records.iter().find(|r| r.name.eq_ignore_ascii_case(&query.name)).ok_or("Unsupported name: this positive-A snapshot supplies no existence or negative-response facts for that name. No NXDOMAIN, NODATA or external lookup is inferred.")?;
        let path = world.probe(query.server);
        if !path.pass {
            return Ok(Observation {
                tool: query.tool,
                server: query.server,
                name: query.name,
                address: None,
                path_reason: path.reason,
            });
        }
        Ok(Observation {
            tool: query.tool,
            server: query.server,
            name: query.name,
            address: Some(answer.address),
            path_reason: path.reason,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Dig,
    Host,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub tool: Tool,
    pub server: Ipv4Addr,
    /// Normalized without the required final root dot, matching supplied facts.
    pub name: String,
}
impl Query {
    pub fn parse(input: &str) -> Result<Self, String> {
        if input.len() > MAX_COMMAND
            || !input.is_ascii()
            || input.bytes().any(|b| b.is_ascii_control() && b != b'\t')
        {
            return Err(format!("Unsupported direct query. {HELP}"));
        }
        let parts: Vec<_> = input.split_whitespace().collect();
        let (tool, server, name, kind) = match parts.as_slice() {
            ["dig", server, name, kind, "+short"] | ["dig", "+short", server, name, kind] => {
                (Tool::Dig, *server, *name, *kind)
            }
            ["dig", server, name, "-t", kind, "+short"]
            | ["dig", "+short", server, name, "-t", kind] => (Tool::Dig, *server, *name, *kind),
            ["host", "-t", kind, name, server] | ["host", "-4", "-t", kind, name, server] => {
                (Tool::Host, *server, *name, *kind)
            }
            _ => return Err(format!("Unsupported syntax. {HELP}")),
        };
        if !kind.eq_ignore_ascii_case("A") {
            return Err("Unsupported query type: these supplied snapshots contain positive A records only; other record types and negative DNS responses are not inferred.".into());
        }
        let raw_server = match tool {
            Tool::Dig => server.strip_prefix('@').ok_or("Use @ followed by a supplied numeric IPv4 resolver; implicit or named servers are unsupported.")?,
            Tool::Host => server,
        };
        let server: Ipv4Addr = raw_server.parse().map_err(|_| "Use a supplied canonical numeric IPv4 resolver; this subset does not look up server names.")?;
        if server.to_string() != raw_server {
            return Err("Use a supplied server in canonical four-octet decimal IPv4 form.".into());
        }
        let name = name.strip_suffix('.').ok_or("Use the full supplied name with a final dot, such as service.lab.test.; this direct-query subset does not apply a search list.")?;
        if name.len() > 80
            || !name.to_ascii_lowercase().ends_with(".lab.test")
            || name.split('.').any(|l| {
                l.is_empty()
                    || l.len() > 63
                    || l.starts_with('-')
                    || l.ends_with('-')
                    || !l.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            })
        {
            return Err("Use one supplied absolute .lab.test name; paths, URLs, escaped names and shell syntax are unsupported.".into());
        }
        Ok(Self {
            tool,
            server,
            name: name.to_ascii_lowercase(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub tool: Tool,
    pub server: Ipv4Addr,
    pub name: String,
    pub address: Option<Ipv4Addr>,
    pub path_reason: String,
}
impl Observation {
    pub fn output(&self) -> String {
        match self.address {
            Some(ip) => {
                let stdout = match self.tool {
                    Tool::Dig => format!("{ip}\n"),
                    Tool::Host => format!(
                        "Using domain server:\nName: {}\nAddress: {}#53\nAliases: \n\n{} has address {ip}\n",
                        self.server, self.server, self.name
                    ),
                };
                format!(
                    "{stdout}\n[Simulator: A answer from {}; command exit status 0; client settings unchanged.]",
                    self.server
                )
            }
            None => format!(
                "[Simulator: no reply evaluated from {}: {}. This is a path observation, not a real utility timeout or DNS response; no utility exit status is inferred.]",
                self.server, self.path_reason
            ),
        }
    }
}
