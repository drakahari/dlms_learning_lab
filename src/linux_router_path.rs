//! Fixed IPv4 path facts plus bounded filter evaluation, not a network stack.
use crate::linux_filter::{Chain, Cidr, Class, Fault, Packet, Protocol, Table, Verdict};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub name: String,
    pub address: Ipv4Addr,
    pub prefix: u8,
    pub up: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub destination: Cidr,
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub device: String,
    pub via: Option<Ipv4Addr>,
    pub metric: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Link,8>")]
    pub links: Vec<Link>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Route,32>")]
    pub routes: Vec<Route>,
}
fn fail<T>(message: &str) -> Result<T, Fault> {
    Err(Fault {
        class: Class::Context,
        message: message.into(),
    })
}
fn device(s: &str) -> bool {
    !s.is_empty()
        && s != "."
        && s != ".."
        && s.len() <= 15
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
impl Link {
    fn network(&self) -> Result<Cidr, Fault> {
        Cidr::parse(&format!("{}/{}", self.address, self.prefix))
    }
}
impl Facts {
    pub fn validate(&self) -> Result<(), Fault> {
        if self.links.is_empty() || self.links.len() > 8 || self.routes.len() > 32 {
            return fail("Supply1to8fixed links and at most32static routes.");
        }
        let (mut names, mut addresses) = (BTreeSet::new(), BTreeSet::new());
        for l in &self.links {
            if !device(&l.name)
                || !names.insert(&l.name)
                || !addresses.insert(l.address)
                || l.address.is_unspecified()
                || l.address.is_multicast()
                || l.address.is_broadcast()
                || l.address.is_loopback()
                || l.address.is_link_local()
            {
                return fail(
                    "Fixed links need unique supported names and unique unicast addresses.",
                );
            }
            l.network()?;
        }
        for (i, r) in self.routes.iter().enumerate() {
            if self.routes[..i].contains(r) {
                return fail("Do not duplicate static route entries.");
            }
            if !r.destination.valid() {
                return fail("Stored route prefixes must be canonical numeric IPv4 networks.");
            }
            let Some(l) = self.links.iter().find(|l| l.name == r.device) else {
                return fail("A static route names an unknown supplied interface.");
            };
            if let Some(g) = r.via
                && (!l.network()?.contains(g)
                    || self.links.iter().any(|l| l.address == g)
                    || g.is_unspecified()
                    || g.is_multicast()
                    || g.is_broadcast()
                    || g.is_loopback()
                    || g.is_link_local())
            {
                return fail(
                    "A next hop must be a nonlocal unicast address on the named supplied link.",
                );
            }
        }
        let routes = self.all_routes()?;
        for (i, a) in routes.iter().enumerate() {
            for b in &routes[i + 1..] {
                if a.destination == b.destination && a.metric == b.metric && a != b {
                    return fail(
                        "Equal-preference multipath routes are outside this fixed routing profile.",
                    );
                }
            }
        }
        Ok(())
    }
    fn all_routes(&self) -> Result<Vec<Route>, Fault> {
        let mut routes = self.routes.clone();
        for l in &self.links {
            let route = Route {
                destination: l.network()?,
                device: l.name.clone(),
                via: None,
                metric: 0,
            };
            if !routes.contains(&route) {
                routes.push(route);
            }
        }
        Ok(routes)
    }
    fn route(&self, ip: Ipv4Addr) -> Result<Option<Route>, Fault> {
        self.validate()?;
        let mut routes = self.all_routes()?;
        routes.retain(|r| r.destination.contains(ip));
        routes.sort_by_key(|r| (std::cmp::Reverse(r.destination.prefix), r.metric));
        Ok(routes.into_iter().next())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Incoming(#[serde(deserialize_with = "crate::cron::text::<_,15>")] String),
    Local,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub source: Ipv4Addr,
    pub destination: Ipv4Addr,
    pub protocol: Protocol,
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Gate {
    Allowed,
    IngressDown,
    NoRoute,
    EgressDown,
    ForwardingDisabled,
    FilterDrop,
    PeerUnavailable,
}
/// Allowed means this modeled path/filter permits the packet, not a successful application exchange.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    pub gate: Gate,
    pub chain: Option<Chain>,
    pub route: Option<Route>,
    pub rule: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub filter: Table,
    pub forwarding: bool,
}
impl Runtime {
    pub fn evaluate(&self, facts: &Facts, origin: &Origin, h: &Header) -> Result<Trace, Fault> {
        facts.validate()?;
        self.filter.validate()?;
        if h.source.is_unspecified()
            || h.destination.is_unspecified()
            || h.source.is_multicast()
            || h.destination.is_multicast()
            || h.source.is_broadcast()
            || h.destination.is_broadcast()
            || h.source.is_loopback()
            || h.destination.is_loopback()
            || h.source.is_link_local()
            || h.destination.is_link_local()
        {
            return Err(Fault{class:Class::Unsupported,message:"This first path profile uses supplied unicast endpoints, without multicast, broadcast, loopback or link-local forwarding.".into()});
        }
        if match h.protocol {
            Protocol::Icmp => h.source_port.is_some() || h.destination_port.is_some(),
            Protocol::Tcp | Protocol::Udp => {
                h.source_port.is_none() || h.destination_port.is_none()
            }
        } {
            return fail(
                "TCP/UDP headers need both numeric ports; ICMP headers have no TCP/UDP ports.",
            );
        }
        let ingress = if let Origin::Incoming(name) = origin {
            let Some(link) = facts.links.iter().find(|l| l.name == *name) else {
                return fail("The packet ingress is not one of the supplied links.");
            };
            if !link.up {
                return Ok(Trace {
                    gate: Gate::IngressDown,
                    chain: None,
                    route: None,
                    rule: None,
                });
            }
            Some(name.clone())
        } else {
            if !facts.links.iter().any(|l| l.address == h.source) {
                return fail(
                    "Local-origin fixtures use one of this router's supplied addresses; raw/freebind sockets are unmodeled.",
                );
            }
            None
        };
        let local_destination = facts.links.iter().any(|l| l.address == h.destination);
        if ingress.is_none() && local_destination {
            return Err(Fault{class:Class::Unsupported,message:"Local-to-local loopback traverses additional hooks and is outside this first path profile.".into()});
        }
        let (chain, route, egress) = if local_destination {
            (Chain::Input, None, None)
        } else {
            let Some(route) = facts.route(h.destination)? else {
                return Ok(Trace {
                    gate: Gate::NoRoute,
                    chain: None,
                    route: None,
                    rule: None,
                });
            };
            let link = facts.links.iter().find(|l| l.name == route.device).unwrap();
            let chain = if ingress.is_some() {
                Chain::Forward
            } else {
                Chain::Output
            };
            if !link.up {
                return Ok(Trace {
                    gate: Gate::EgressDown,
                    chain: Some(chain),
                    route: Some(route),
                    rule: None,
                });
            }
            if chain == Chain::Forward && !self.forwarding {
                return Ok(Trace {
                    gate: Gate::ForwardingDisabled,
                    chain: Some(chain),
                    route: Some(route),
                    rule: None,
                });
            }
            let device = route.device.clone();
            (chain, Some(route), Some(device))
        };
        let decision = self.filter.evaluate(
            chain,
            &Packet {
                source: h.source,
                destination: h.destination,
                ingress,
                egress,
                protocol: h.protocol,
                source_port: h.source_port,
                destination_port: h.destination_port,
            },
        )?;
        Ok(Trace {
            gate: if decision.verdict == Verdict::Accept {
                Gate::Allowed
            } else {
                Gate::FilterDrop
            },
            chain: Some(chain),
            route,
            rule: decision.rule,
        })
    }
}
