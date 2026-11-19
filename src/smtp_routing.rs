//! Finite mail-route observations, not SMTP sockets or a running MTA.
use crate::{
    local_mail,
    smtp_config::{self as config, Config, ErrorKind, Fault, RelayDecision, RelayTarget},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};

pub const MAX_FACTS: usize = 128 * 1024;
pub const MAX_NODES: usize = 6;
pub const MAX_ROUTES: usize = 16;
pub const MAX_HOPS: usize = 16;

fn bad(message: impl Into<String>) -> Fault {
    Fault {
        kind: ErrorKind::Invalid,
        message: message.into(),
    }
}
fn identifier(value: &str) -> bool {
    local_mail::name(value) && value == value.to_ascii_lowercase()
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub hostname: String,
    pub address: Ipv4Addr,
    pub port: u16,
    pub available: bool,
    pub users: BTreeSet<String>,
    pub aliases: String,
    pub forwards: BTreeMap<String, String>,
}
impl Node {
    pub fn directory(&self) -> Result<local_mail::Directory, Fault> {
        if self.aliases.len() > 4096
            || self.forwards.len() > 8
            || self.forwards.values().any(|s| s.len() > 1024)
        {
            return Err(bad(
                "Keep supplied local aliases within4KiB and at most8forward files within1KiB each.",
            ));
        }
        let directory = local_mail::Directory {
            users: self.users.clone(),
            aliases: local_mail::parse_aliases(&self.aliases).map_err(bad)?,
            forwards: self
                .forwards
                .iter()
                .map(|(name, text)| {
                    Ok((name.clone(), local_mail::parse_forward(text).map_err(bad)?))
                })
                .collect::<Result<_, Fault>>()?,
        };
        directory.validate().map_err(bad)?;
        Ok(directory)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub nodes: BTreeMap<String, Node>,
    /// Supplied literal next hops. These are NOT a DNS/MX query implementation.
    pub routes: BTreeMap<String, String>,
}
fn fits<T: Serialize>(value: &T) -> bool {
    struct Budget(usize);
    impl std::io::Write for Budget {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .filter(|n| *n <= MAX_FACTS)
                .ok_or_else(|| std::io::Error::other("Mail-source size limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Budget(0), value).is_ok()
}
impl Facts {
    pub fn validate(&self) -> Result<(), Fault> {
        if !fits(self)
            || !(1..=MAX_NODES).contains(&self.nodes.len())
            || self.routes.len() > MAX_ROUTES
        {
            return Err(Fault {
                kind: ErrorKind::Limit,
                message: "Supply at most6mail servers,16literal routes and128KiB of source facts."
                    .into(),
            });
        }
        let mut hosts = BTreeSet::new();
        let mut addresses = BTreeSet::new();
        for (id, node) in &self.nodes {
            if !identifier(id)
                || config::domain(&node.hostname)? != node.hostname
                || node.port == 0
                || node.address.is_unspecified()
                || node.address.is_multicast()
                || node.address == Ipv4Addr::BROADCAST
                || !hosts.insert(&node.hostname)
                || !addresses.insert(node.address)
            {
                return Err(bad(
                    "Supplied servers need unique canonical hostnames/IPv4 addresses, valid ports and simple stable identifiers.",
                ));
            }
            node.directory()?;
        }
        for (domain, target) in &self.routes {
            if config::domain(domain)? != *domain
                || self.endpoint(&RelayTarget::parse(target)?).is_none()
            {
                return Err(bad(
                    "Each supplied literal mail route needs a canonical domain and an existing represented endpoint.",
                ));
            }
        }
        Ok(())
    }
    fn endpoint(&self, target: &RelayTarget) -> Option<&str> {
        self.nodes
            .iter()
            .find(|(_, n)| {
                n.port == target.port
                    && (n.hostname == target.host
                        || target.host.parse::<Ipv4Addr>().ok() == Some(n.address))
            })
            .map(|(id, _)| id.as_str())
    }
    /// Parse one bounded active snapshot for a group of message observations.
    pub fn prepare(&self, active: &BTreeMap<String, String>) -> Result<Prepared<'_>, Fault> {
        self.validate()?;
        if active.keys().ne(self.nodes.keys()) {
            return Err(bad(
                "Use exactly one active configuration per represented server.",
            ));
        }
        let parsed = self
            .nodes
            .keys()
            .map(|id| (id.as_str(), Config::parse(&active[id])))
            .collect();
        Ok(Prepared {
            facts: self,
            parsed,
        })
    }
    /// New SMTP admission: each server sees its immediate peer.
    pub fn trace(
        &self,
        active: &BTreeMap<String, String>,
        entry: &str,
        client: Ipv4Addr,
        recipient: &str,
    ) -> Result<Trace, Fault> {
        self.prepare(active)?.incoming(entry, client, recipient)
    }
    /// Previously accepted work skips a new SMTP admission at its queue owner.
    pub fn trace_queued(
        &self,
        active: &BTreeMap<String, String>,
        entry: &str,
        recipient: &str,
    ) -> Result<Trace, Fault> {
        self.prepare(active)?.queued(entry, recipient)
    }
}
/// Immutable validated facts and parsed configurations, reused during one bounded run.
pub struct Prepared<'a> {
    facts: &'a Facts,
    parsed: BTreeMap<&'a str, Result<Config, Fault>>,
}
impl Prepared<'_> {
    pub fn incoming(&self, entry: &str, client: Ipv4Addr, recipient: &str) -> Result<Trace, Fault> {
        self.walk(entry, Some(client), recipient)
    }
    pub fn queued(&self, entry: &str, recipient: &str) -> Result<Trace, Fault> {
        self.walk(entry, None, recipient)
    }
    fn walk(&self, entry: &str, client: Option<Ipv4Addr>, recipient: &str) -> Result<Trace, Fault> {
        if !self.facts.nodes.contains_key(entry) {
            return Err(bad("Choose a supplied entry server."));
        }
        let recipient = Recipient::parse(recipient)?;
        let mut current = entry.to_owned();
        let mut peer = client;
        let mut visited = BTreeSet::new();
        let mut hops = vec![];
        for _ in 0..MAX_HOPS {
            if !visited.insert((current.clone(), peer)) {
                return Ok(Trace {
                    hops,
                    outcome: Outcome::Loop {
                        repeats: current,
                        client: peer,
                    },
                });
            }
            let node = &self.facts.nodes[&current];
            let mut hop = Hop {
                node: current.clone(),
                client: peer,
                action: HopAction::Entered,
            };
            if !node.available {
                hop.action = HopAction::Unavailable;
                hops.push(hop);
                return Ok(Trace {
                    hops,
                    outcome: Outcome::Unavailable {
                        target: node.hostname.clone(),
                        port: node.port,
                    },
                });
            }
            let cfg = match &self.parsed[current.as_str()] {
                Ok(cfg) => cfg,
                Err(error) => {
                    hop.action = HopAction::ConfigurationProblem;
                    hops.push(hop);
                    return Ok(Trace {
                        hops,
                        outcome: Outcome::ConfigurationProblem {
                            node: current,
                            error: error.clone(),
                        },
                    });
                }
            };
            match peer
                .map(|ip| cfg.relay_decision(ip, &recipient.domain))
                .transpose()?
                .unwrap_or(RelayDecision::Permit)
            {
                RelayDecision::Reject => {
                    hop.action = HopAction::Rejected;
                    hops.push(hop);
                    return Ok(Trace {
                        hops,
                        outcome: Outcome::Rejected { node: current },
                    });
                }
                RelayDecision::Defer => {
                    hop.action = HopAction::Deferred;
                    hops.push(hop);
                    return Ok(Trace {
                        hops,
                        outcome: Outcome::Deferred { node: current },
                    });
                }
                RelayDecision::Permit => {}
            }
            if cfg.destinations.contains(&recipient.domain) {
                let delivery = node.directory()?.deliver(&recipient.local).map_err(bad)?;
                hop.action = HopAction::Local;
                hops.push(hop);
                return Ok(Trace {
                    hops,
                    outcome: Outcome::Local {
                        node: current,
                        mailboxes: delivery.mailboxes,
                        failures: delivery.failures,
                    },
                });
            }
            let target = match cfg.relay.clone() {
                Some(target) => Some(target),
                None => self
                    .facts
                    .routes
                    .get(&recipient.domain)
                    .map(|s| RelayTarget::parse(s))
                    .transpose()?,
            };
            let Some(target) = target else {
                hop.action = HopAction::NoRoute;
                hops.push(hop);
                return Ok(Trace {
                    hops,
                    outcome: Outcome::NoRoute { node: current },
                });
            };
            let Some(next) = self.facts.endpoint(&target) else {
                hop.action = HopAction::Unavailable;
                hops.push(hop);
                return Ok(Trace {
                    hops,
                    outcome: Outcome::Unavailable {
                        target: target.host,
                        port: target.port,
                    },
                });
            };
            hop.action = HopAction::Relay { to: next.into() };
            hops.push(hop);
            peer = Some(node.address);
            current = next.into();
        }
        Ok(Trace {
            hops,
            outcome: Outcome::Limit,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipient {
    pub local: String,
    pub domain: String,
}
impl Recipient {
    pub fn parse(text: &str) -> Result<Self, Fault> {
        if text.len() > 320 {
            return Err(Fault {
                kind: ErrorKind::Limit,
                message: "Keep the sample recipient within320bytes.".into(),
            });
        }
        let Some((local, domain)) = text.split_once('@') else {
            return Err(bad("Write one sample recipient as local-name@domain.test."));
        };
        if !identifier(local) {
            return Err(Fault { kind: ErrorKind::Unsupported, message: "Use a supplied lower-case local recipient. Quoted/extended names and address lists are outside this mail subset.".into() });
        }
        Ok(Self {
            local: local.into(),
            domain: config::domain(domain)?,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hop {
    pub node: String,
    pub client: Option<Ipv4Addr>,
    pub action: HopAction,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HopAction {
    Entered,
    Relay { to: String },
    Local,
    Rejected,
    Deferred,
    Unavailable,
    ConfigurationProblem,
    NoRoute,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Local {
        node: String,
        mailboxes: BTreeSet<String>,
        failures: BTreeSet<String>,
    },
    Rejected {
        node: String,
    },
    Deferred {
        node: String,
    },
    NoRoute {
        node: String,
    },
    Unavailable {
        target: String,
        port: u16,
    },
    ConfigurationProblem {
        node: String,
        error: Fault,
    },
    /// A finite configuration-path finding, not an invented SMTP bounce code.
    Loop {
        repeats: String,
        client: Option<Ipv4Addr>,
    },
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    pub hops: Vec<Hop>,
    pub outcome: Outcome,
}
