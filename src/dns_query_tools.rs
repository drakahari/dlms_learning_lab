//! Read-only, nonrecursive utility projection of existing authoritative DNS state.
//! No file, socket, resolver, subprocess, timer or external lookup is used.
use crate::{
    dns::{self, Answer, Code, Data, QueryType, Server, Zone},
    host_direct_dns::Tool,
};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

pub const MAX_COMMAND: usize = 512;
pub const MAX_OUTPUT: usize = 32_768;
pub const HELP: &str = "Supported authoritative-query subset: dig @SERVER NAME. TYPE +norecurse +short (also -t TYPE; +short may lead), or host -r -t TYPE NAME. SERVER (optional -4 before -r). TYPE is A, AAAA, CNAME, MX, NS or PTR. Use an absolute ASCII name and a supplied numeric IPv4 server. These query the published primary/secondary copies only; +norecurse and -r disable recursion. No implicit server/type, default multi-type/reverse conversion, shell syntax, files, search, cache, DNSSEC, transport, TTL/header output or other flags. Responses derive from the selected typed authoritative zone, never the host network.";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Served {
    Primary,
    Secondary,
}
impl Served {
    pub fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
        }
    }
    pub fn zone(self, server: &Server) -> &Zone {
        match self {
            Self::Primary => &server.primary,
            Self::Secondary => &server.secondary,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub address: Ipv4Addr,
    pub served: Served,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Endpoint,2>")]
    pub endpoints: Vec<Endpoint>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub assignment: Plan,
    pub example: Plan,
}
impl Plan {
    pub fn validate(&self, server: &Server) -> Result<(), String> {
        server.validate().map_err(|e| e.message)?;
        if self.endpoints.is_empty() || self.endpoints.len() > 2 {
            return Err("Supply one or two explicit published DNS endpoints.".into());
        }
        for (i, e) in self.endpoints.iter().enumerate() {
            if !crate::ipv4_routing::supported_address(e.address)
                || self.endpoints[..i]
                    .iter()
                    .any(|p| p.address == e.address || p.served == e.served)
            {
                return Err("DNS endpoints need distinct private/documentation IPv4 addresses and distinct published copies.".into());
            }
        }
        Ok(())
    }
    pub fn observe(&self, server: &Server, input: &str) -> Result<Observation, String> {
        self.validate(server)?;
        let question = Question::parse(input)?;
        let endpoint=self.endpoints.iter().find(|e|e.address==question.server).ok_or("Unsupported server: choose a supplied published DNS endpoint; no external lookup was attempted.")?;
        let answer = endpoint
            .served
            .zone(server)
            .query(&question.name, question.kind)
            .map_err(|e| e.message)?;
        Ok(Observation {
            question,
            served: endpoint.served,
            answer,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    pub tool: Tool,
    pub server: Ipv4Addr,
    pub original_name: String,
    pub name: String,
    pub kind: QueryType,
}
impl Question {
    pub fn parse(input: &str) -> Result<Self, String> {
        if input.len() > MAX_COMMAND
            || !input.is_ascii()
            || input.bytes().any(|b| b.is_ascii_control() && b != b'\t')
        {
            return Err(format!("Unsupported query input. {HELP}"));
        }
        let p: Vec<_> = input.split_whitespace().collect();
        let (tool, server, name, kind) = match p.as_slice() {
            ["dig", s, n, k, "+norecurse", "+short"]
            | ["dig", s, n, k, "+short", "+norecurse"]
            | ["dig", "+short", s, n, k, "+norecurse"] => (Tool::Dig, *s, *n, *k),
            ["dig", s, n, "-t", k, "+norecurse", "+short"]
            | ["dig", s, n, "-t", k, "+short", "+norecurse"]
            | ["dig", "+short", s, n, "-t", k, "+norecurse"] => (Tool::Dig, *s, *n, *k),
            ["host", "-r", "-t", k, n, s]
            | ["host", "-4", "-r", "-t", k, n, s]
            | ["host", "-r", "-4", "-t", k, n, s] => (Tool::Host, *s, *n, *k),
            _ => return Err(format!("Unsupported syntax. {HELP}")),
        };
        let raw = match tool {
            Tool::Dig => server
                .strip_prefix('@')
                .ok_or("Use @ followed by a supplied numeric server for dig.")?,
            Tool::Host => server,
        };
        let server: Ipv4Addr = raw.parse().map_err(
            |_| "Use a canonical numeric IPv4 server; server-name lookups are unsupported.",
        )?;
        if server.to_string() != raw {
            return Err("Use canonical four-octet decimal IPv4 for the server.".into());
        }
        if !name.ends_with('.') {
            return Err(
                "Use an absolute name ending in a dot; no search suffix is added in this activity."
                    .into(),
            );
        }
        let canonical = dns::canonical_name(name).map_err(|e| e.message)?;
        let kind=match kind.to_ascii_uppercase().as_str(){"A"=>QueryType::A,"AAAA"=>QueryType::Aaaa,"CNAME"=>QueryType::Cname,"MX"=>QueryType::Mx,"NS"=>QueryType::Ns,"PTR"=>QueryType::Ptr,_=>return Err("Unsupported record type: choose A, AAAA, CNAME, MX, NS or PTR; other types are not approximated.".into())};
        Ok(Self {
            tool,
            server,
            original_name: name.into(),
            name: canonical,
            kind,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub question: Question,
    pub served: Served,
    pub answer: Answer,
}
pub fn code_text(code: Code) -> &'static str {
    match code {
        Code::Noerror => "NOERROR",
        Code::Nxdomain => "NXDOMAIN",
        Code::Refused => "REFUSED",
        Code::Servfail => "SERVFAIL",
    }
}
fn push(out: &mut String, text: &str) -> Result<(), String> {
    if out.len().saturating_add(text.len()) > MAX_OUTPUT {
        return Err("The full lookup result exceeds this simulator's 32768-byte output limit; no truncated result is reported as complete.".into());
    }
    out.push_str(text);
    Ok(())
}
impl Observation {
    /// Utility status is distinct from DNS response code or objective success.
    pub fn status(&self) -> u8 {
        match self.question.tool {
            Tool::Dig => 0,
            Tool::Host => u8::from(self.answer.code != Code::Noerror),
        }
    }
    pub fn output(&self) -> Result<String, String> {
        let mut out = String::new();
        match self.question.tool {
            Tool::Dig => {
                for record in &self.answer.records {
                    push(
                        &mut out,
                        &format!("{}\n", crate::dns_workspace::data_text(&record.data)),
                    )?;
                }
            }
            Tool::Host => {
                push(
                    &mut out,
                    &format!(
                        "Using domain server:\nName: {}\nAddress: {}#53\nAliases: \n\n",
                        self.question.server, self.question.server
                    ),
                )?;
                if self.answer.code != Code::Noerror {
                    let number = match self.answer.code {
                        Code::Nxdomain => 3,
                        Code::Refused => 5,
                        Code::Servfail => 2,
                        Code::Noerror => unreachable!(),
                    };
                    let name = if self.answer.code == Code::Nxdomain {
                        self.question.original_name.as_str()
                    } else {
                        self.question.name.trim_end_matches('.')
                    };
                    push(
                        &mut out,
                        &format!(
                            "Host {name} not found: {number}({})\n",
                            code_text(self.answer.code)
                        ),
                    )?;
                } else if self.answer.records.is_empty() {
                    push(
                        &mut out,
                        &format!(
                            "{} has no {} record\n",
                            self.question.name.trim_end_matches('.'),
                            crate::dns_workspace::type_text(self.question.kind)
                        ),
                    )?;
                } else {
                    for record in &self.answer.records {
                        let phrase = match &record.data {
                            Data::A { .. } => "has address",
                            Data::Aaaa { .. } => "has IPv6 address",
                            Data::Cname { .. } => "is an alias for",
                            Data::Mx { .. } => "mail is handled by",
                            Data::Ns { .. } => "name server",
                            Data::Ptr { .. } => "domain name pointer",
                        };
                        push(
                            &mut out,
                            &format!(
                                "{} {phrase} {}\n",
                                record.owner.trim_end_matches('.'),
                                crate::dns_workspace::data_text(&record.data)
                            ),
                        )?;
                    }
                }
            }
        }
        push(
            &mut out,
            &format!(
                "\n[Simulator: {} copy; DNS {}; command exit status {}]\n{}",
                self.served.label(),
                code_text(self.answer.code),
                self.status(),
                self.answer.note
            ),
        )?;
        Ok(out)
    }
}
