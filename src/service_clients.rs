//! Virtual client targets resolved from supplied text, using the existing reducer.
//! No host lookup, sockets, files, commands, clock or application protocol runs.
use crate::{service_names as names, transport_core as net};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};
type Result<T> = std::result::Result<T, names::Fault>;
fn fail(kind: names::FaultKind, message: impl Into<String>) -> names::Fault {
    names::Fault {
        kind,
        line: None,
        message: message.into(),
    }
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Listener {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub role: String,
    pub protocol: net::Protocol,
    pub port: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub version: u8,
    pub client: Ipv4Addr,
    pub server: Ipv4Addr,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Listener,8>")]
    pub listeners: Vec<Listener>,
}
impl Profile {
    pub fn validate(&self) -> Result<()> {
        let bad_ip = |ip: Ipv4Addr| ip.is_unspecified() || ip.is_multicast() || ip.is_broadcast();
        if self.version != 1
            || self.client == self.server
            || bad_ip(self.client)
            || bad_ip(self.server)
        {
            return Err(fail(
                names::FaultKind::Invalid,
                "A version 1 virtual profile needs two distinct supplied unicast IPv4 hosts.",
            ));
        }
        if self.listeners.is_empty() || self.listeners.len() > net::MAX_LISTENERS {
            return Err(fail(
                names::FaultKind::Limit,
                "Supply between 1 and 8 virtual server listeners.",
            ));
        }
        let mut roles = BTreeSet::new();
        let mut sockets = BTreeSet::new();
        for l in &self.listeners {
            if !id(&l.role)
                || !roles.insert(&l.role)
                || l.port == 0
                || !sockets.insert((names::protocol_name(l.protocol), l.port))
            {
                return Err(fail(
                    names::FaultKind::Invalid,
                    "Server roles and protocol/port pairs must be distinct, with bounded identifiers and nonzero ports.",
                ));
            }
        }
        Ok(())
    }
    pub fn listener(&self, role: &str) -> Option<&Listener> {
        self.listeners.iter().find(|l| l.role == role)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Resolved {
    pub protocol: net::Protocol,
    pub port: u16,
    /// Numeric targets bypass the name database; never invent a canonical name.
    pub canonical_name: Option<String>,
}
pub fn resolve(database: &str, protocol: net::Protocol, setting: &str) -> Result<Resolved> {
    if setting.len() > 64 {
        return Err(fail(
            names::FaultKind::Limit,
            "Keep a client service setting within 64 bytes.",
        ));
    }
    let setting = setting.trim();
    if setting.is_empty() {
        return Err(fail(
            names::FaultKind::Syntax,
            "Enter a service name or decimal port for this client.",
        ));
    }
    if setting.contains('/') {
        return Err(fail(
            names::FaultKind::Syntax,
            "Keep the transport in its separate setting. This service field accepts a name or decimal port, without /tcp or /udp.",
        ));
    }
    if setting.bytes().all(|b| b.is_ascii_digit()) {
        let port=setting.parse::<u16>().ok().filter(|p|*p!=0).ok_or_else(||fail(names::FaultKind::Invalid,"A remote client target needs a decimal port from 1 to 65535; port 0 is not a usable remote service here."))?;
        return Ok(Resolved {
            protocol,
            port,
            canonical_name: None,
        });
    }
    let query = names::Query::parse(&format!("{setting}/{}", names::protocol_name(protocol)))?;
    let data = names::Database::parse(database)?;
    let record=data.lookup(&query).ok_or_else(||fail(names::FaultKind::Invalid,format!("No supplied service entry matches {setting}/{}. Names and aliases are case-sensitive and protocol-specific; inspect the virtual file and supplied server listeners.",names::protocol_name(protocol))))?;
    if record.port == 0 {
        return Err(fail(
            names::FaultKind::Invalid,
            "The database maps this name to port 0. That reserved value cannot be this exercise's remote client target.",
        ));
    }
    Ok(Resolved {
        protocol,
        port: record.port,
        canonical_name: Some(record.name.clone()),
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub resolved: Resolved,
    pub transport: net::Report,
}
/// Compute actual transport behavior independently of any later grading target.
/// A delivered opaque payload is not evidence of DNS/TLS/SSH interoperability.
pub fn run(
    profile: &Profile,
    database: &str,
    protocol: net::Protocol,
    service: &str,
    payload: &str,
    disabled_roles: &[String],
) -> Result<Report> {
    profile.validate()?;
    if disabled_roles.len() > 8 || payload.len() > net::MAX_PAYLOAD_BYTES {
        return Err(fail(
            names::FaultKind::Limit,
            "The supplied request exceeds the bounded listener/payload limits.",
        ));
    }
    let mut disabled = BTreeSet::new();
    for role in disabled_roles {
        if !id(role) || profile.listener(role).is_none() || !disabled.insert(role.as_str()) {
            return Err(fail(
                names::FaultKind::Invalid,
                "Only distinct supplied listener roles may be disabled in a source-held test case.",
            ));
        }
    }
    let resolved = resolve(database, protocol, service)?;
    let config = net::Config {
        version: net::VERSION,
        hosts: vec![
            net::Host {
                id: "client".into(),
                address: profile.client,
            },
            net::Host {
                id: "server".into(),
                address: profile.server,
            },
        ],
        listeners: profile
            .listeners
            .iter()
            .filter(|l| !disabled.contains(l.role.as_str()))
            .map(|l| net::Listener {
                protocol: l.protocol,
                endpoint: net::Endpoint {
                    host: "server".into(),
                    port: l.port,
                },
            })
            .collect(),
        exchanges: vec![net::Exchange {
            id: "request".into(),
            protocol,
            client: net::Endpoint {
                host: "client".into(),
                port: 40000,
            },
            server: net::Endpoint {
                host: "server".into(),
                port: resolved.port,
            },
            payload: payload.into(),
        }],
    };
    let transport = net::run(&config, "request", &mut |_| {
        Ok(net::Decision::delivered(
            "The supplied virtual path delivers this modeled packet.",
        ))
    })
    .map_err(|e| fail(names::FaultKind::Invalid, e.message))?;
    Ok(Report {
        resolved,
        transport,
    })
}

#[cfg(test)]
mod tests;
