//! Fixed IPv6 path facts plus bounded filter evaluation, not a network stack.
use crate::linux_v6_filter::{Chain, Cidr, Class, Fault, Packet, Protocol, Table, Verdict};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv6Addr};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub name: String,
    pub address: Ipv6Addr,
    pub link_local: Ipv6Addr,
    pub prefix: u8,
    pub up: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub destination: Cidr,
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub device: String,
    pub via: Option<Ipv6Addr>,
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
                || !ordinary(l.address)
                || crate::ipv6::category(l.link_local) != crate::ipv6::Category::LinkLocal
            {
                return fail(
                    "Fixed links need unique names/ordinary unicast addresses and one explicitly scoped link-local address per link.",
                );
            }
            l.network()?;
        }
        for (i, r) in self.routes.iter().enumerate() {
            if self.routes[..i].contains(r) {
                return fail("Do not duplicate static route entries.");
            }
            if !r.destination.valid() {
                return fail("Stored route prefixes must be canonical numeric IPv6 networks.");
            }
            let Some(l) = self.links.iter().find(|l| l.name == r.device) else {
                return fail("A static route names an unknown supplied interface.");
            };
            if let Some(g) = r.via {
                let link_local = crate::ipv6::category(g) == crate::ipv6::Category::LinkLocal;
                if (link_local && g == l.link_local)
                    || (!link_local
                        && (!ordinary(g)
                            || !l.network()?.contains(g)
                            || self.links.iter().any(|l| l.address == g)))
                {
                    return fail(
                        "A next hop must be a distinct unicast neighbor on the named link. A link-local next hop uses that route's explicit interface scope.",
                    );
                }
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
    pub(crate) fn all_routes(&self) -> Result<Vec<Route>, Fault> {
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
    pub(crate) fn route(&self, ip: Ipv6Addr) -> Result<Option<Route>, Fault> {
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
    pub source: Ipv6Addr,
    pub destination: Ipv6Addr,
    pub protocol: Protocol,
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
    pub icmp_type: Option<u8>,
    pub icmp_code: Option<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gate {
    Allowed,
    IngressDown,
    NoRoute,
    EgressDown,
    ForwardingDisabled,
    FilterDrop,
    ScopeBound,
    TrackingRefused,
}
/// Allowed means this modeled path/filter permits the packet, not a successful application exchange.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
        if ![h.source, h.destination]
            .into_iter()
            .all(|a| ordinary(a) || crate::ipv6::category(a) == crate::ipv6::Category::LinkLocal)
        {
            return Err(Fault{class:Class::Unsupported,message:"This IPv6 path profile excludes unspecified, multicast, mapped and loopback endpoints; no packet or neighbor exchange was invented.".into()});
        }
        if match h.protocol {
            Protocol::Icmp6 => {
                h.source_port.is_some()
                    || h.destination_port.is_some()
                    || h.icmp_type.is_none()
                    || h.icmp_code.is_none()
            }
            Protocol::Tcp | Protocol::Udp => {
                h.source_port.is_none()
                    || h.destination_port.is_none()
                    || h.icmp_type.is_some()
                    || h.icmp_code.is_some()
            }
        } {
            return fail(
                "TCP/UDP headers need two ports and no ICMP fields; ICMPv6 needs numeric type/code and no ports.",
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
            if [h.source, h.destination]
                .into_iter()
                .any(|a| crate::ipv6::category(a) == crate::ipv6::Category::LinkLocal)
            {
                return Err(Fault{class:Class::Unsupported,message:"Local link-local output needs an explicit socket/interface scope, outside this first incoming-traffic profile.".into()});
            }
            if !facts.links.iter().any(|l| l.address == h.source) {
                return fail(
                    "Local-origin fixtures use one of this router's supplied addresses; raw/freebind sockets are unmodeled.",
                );
            }
            None
        };
        let local_destination = facts.links.iter().any(|l| l.address == h.destination)
            || facts.links.iter().any(|l| {
                ingress.as_deref() == Some(l.name.as_str()) && l.link_local == h.destination
            });
        if !local_destination
            && ingress.is_some()
            && [h.source, h.destination]
                .into_iter()
                .any(|a| crate::ipv6::category(a) == crate::ipv6::Category::LinkLocal)
        {
            return Ok(Trace {
                gate: Gate::ScopeBound,
                chain: None,
                route: None,
                rule: None,
            });
        }
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
                icmp_type: h.icmp_type,
                icmp_code: h.icmp_code,
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

fn ordinary(a: Ipv6Addr) -> bool {
    matches!(
        crate::ipv6::category(a),
        crate::ipv6::Category::UniqueLocal
            | crate::ipv6::Category::Documentation
            | crate::ipv6::Category::OtherUnicast
    )
}
