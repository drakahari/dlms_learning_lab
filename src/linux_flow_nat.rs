//! Ordered numeric IPv4 DNAT/SNAT/MASQUERADE, normal supplied TCP/UDP only.
use crate::{
    linux_connections::{self as ct, Transport, Tuple},
    linux_filter::{self as filter, Chain, Protocol, Verdict},
};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
pub const MAX_RULES: usize = 32;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ports {
    pub first: u16,
    pub last: u16,
}
impl Ports {
    fn valid(self) -> bool {
        self.first <= self.last
    }
    fn contains(self, p: u16) -> bool {
        p >= self.first && p <= self.last
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Target {
    Dnat {
        address: Ipv4Addr,
        port: Option<u16>,
    },
    Snat {
        address: Ipv4Addr,
        ports: Option<Ports>,
    },
    Masquerade {
        ports: Option<Ports>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub predicate: filter::Rule,
    pub target: Target,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Table {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Rule,32>")]
    pub prerouting: Vec<Rule>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Rule,32>")]
    pub postrouting: Vec<Rule>,
}
fn ip(a: Ipv4Addr) -> bool {
    !a.is_unspecified()
        && !a.is_multicast()
        && !a.is_broadcast()
        && !a.is_loopback()
        && !a.is_link_local()
}
fn packet(t: Tuple, ingress: Option<&str>, egress: Option<&str>) -> filter::Packet {
    filter::Packet {
        source: t.source.address,
        destination: t.destination.address,
        source_port: Some(t.source.port),
        destination_port: Some(t.destination.port),
        protocol: match t.transport {
            Transport::Tcp => Protocol::Tcp,
            Transport::Udp => Protocol::Udp,
        },
        ingress: ingress.map(str::to_string),
        egress: egress.map(str::to_string),
    }
}
fn single(rule: &Rule, pre: bool) -> filter::Table {
    let mut t = filter::Table::new(Verdict::Drop);
    if pre {
        t.input.rules.push(rule.predicate.clone());
    } else {
        t.output.rules.push(rule.predicate.clone());
    }
    t
}
impl Table {
    pub fn validate(&self) -> Result<(), String> {
        if self.prerouting.len() + self.postrouting.len() > MAX_RULES {
            return Err("Use at most32NAT rules across both supported hooks.".into());
        }
        for (pre, rules) in [(true, &self.prerouting), (false, &self.postrouting)] {
            for r in rules {
                if r.predicate.target != Verdict::Accept
                    || r.predicate.protocol == Some(Protocol::Icmp)
                {
                    return Err("NAT predicates describe supplied TCP/UDP flow matching, not a filter verdict or ICMP mapping.".into());
                }
                single(r, pre).validate().map_err(|e| e.message)?;
                match &r.target{
    Target::Dnat{address,port}if pre&&ip(*address)&& (port.is_none()||r.predicate.protocol.is_some())=>{},
    Target::Snat{address,ports}if !pre&&ip(*address)&&ports.is_none_or(Ports::valid)&&(ports.is_none()||r.predicate.protocol.is_some())=>{},
    Target::Masquerade{ports}if !pre&&ports.is_none_or(Ports::valid)&&(ports.is_none()||r.predicate.protocol.is_some())=>{},
    _=>return Err("Use DNAT only before routing; SNAT/MASQUERADE only after routing, with supported numeric addresses and protocol-bound port changes.".into()),
   }
            }
        }
        Ok(())
    }
    /// First packet only. Existing confirmed flows use the retained connection mapping.
    pub fn destination(
        &self,
        incoming: Tuple,
        ingress: &str,
    ) -> Result<(Tuple, Option<usize>), String> {
        self.validate()?;
        if !incoming.valid() {
            return Err("Supply a supported numeric unicast tuple.".into());
        }
        for (i, r) in self.prerouting.iter().enumerate() {
            if single(r, true)
                .evaluate(Chain::Input, &packet(incoming, Some(ingress), None))
                .map_err(|e| e.message)?
                .verdict
                == Verdict::Accept
            {
                let Target::Dnat { address, port } = r.target else {
                    return Err("Invalid prerouting target".into());
                };
                let mut result = incoming;
                result.destination.address = address;
                if let Some(p) = port {
                    result.destination.port = p;
                }
                if !result.valid() {
                    return Err("This translation leaves the normal-flow tuple profile; no mapping changed.".into());
                }
                return Ok((result, Some(i + 1)));
            }
        }
        Ok((incoming, None))
    }
    /// First accepted path only; returns a candidate without confirming or editing conntrack.
    /// Port allocation is deterministic for teaching, not the kernel's allocation algorithm.
    pub fn source(
        &self,
        forward: Tuple,
        egress: &str,
        address: Ipv4Addr,
        connections: &ct::Table,
    ) -> Result<(Tuple, Option<usize>, Option<String>), String> {
        self.validate()?;
        connections.validate()?;
        if !forward.valid() || !ip(address) {
            return Err(
                "Supply supported numeric tuples and a unicast outgoing interface address.".into(),
            );
        }
        for (i, r) in self.postrouting.iter().enumerate() {
            if single(r, false)
                .evaluate(Chain::Output, &packet(forward, None, Some(egress)))
                .map_err(|e| e.message)?
                .verdict
                == Verdict::Accept
            {
                let (source, range, masq) = match r.target {
                    Target::Snat { address, ports } => (address, ports, None),
                    Target::Masquerade { ports } => (address, ports, Some(egress.to_owned())),
                    _ => return Err("Invalid postrouting target".into()),
                };
                let result = allocate(
                    forward,
                    source,
                    range.unwrap_or_else(|| implicit_ports(forward.source.port)),
                    connections,
                )?;
                return Ok((result, Some(i + 1), masq));
            }
        }
        // Even a null source binding can need an implicit port remap when a
        // previous translation occupies its tuple. NAT is active in this profile.
        let result = allocate(
            forward,
            forward.source.address,
            implicit_ports(forward.source.port),
            connections,
        )?;
        Ok((result, None, None))
    }
}
fn implicit_ports(port: u16) -> Ports {
    match port {
        0..=511 => Ports {
            first: 0,
            last: 511,
        },
        512..=1023 => Ports {
            first: 512,
            last: 1023,
        },
        _ => Ports {
            first: 1024,
            last: 65535,
        },
    }
}
fn allocate(
    forward: Tuple,
    source: Ipv4Addr,
    range: Ports,
    connections: &ct::Table,
) -> Result<Tuple, String> {
    if source == forward.destination.address {
        return Err("Translation to the destination's own address is outside the supplied forwarding profile; no mapping changed.".into());
    }
    let mut result = forward;
    result.source.address = source;
    let available = |port| {
        let mut candidate = result;
        candidate.source.port = port;
        candidate.valid()
            && !connections
                .entries
                .iter()
                .any(|e| e.original == candidate.reverse() || e.wire == candidate)
    };
    let preferred = forward.source.port;
    let chosen = if range.contains(preferred) && available(preferred) {
        Some(preferred)
    } else {
        // Two possible lookup keys per entry, and all candidate address pairs are valid.
        // At most129trials find a free key or exhaust the finite selected range.
        (range.first..=range.last)
            .take(2 * ct::MAX_CONNECTIONS + 1)
            .find(|&port| available(port))
    };
    result.source.port = chosen.ok_or(
        "The selected translated port range has no free tuple; no connection mapping changed",
    )?;
    Ok(result)
}
/// Build predicates through the existing typed filter parser, preserving its supported numeric rules.
pub fn predicate(command: &str, pre: bool) -> Result<filter::Rule, String> {
    let mut t = filter::Table::new(Verdict::Drop);
    t.command(command).map_err(|e| e.message)?;
    let rules = if pre { &t.input.rules } else { &t.output.rules };
    if rules.len() != 1
        || t.input.rules.len() + t.forward.rules.len() + t.output.rules.len() != 1
        || rules[0].target != Verdict::Accept
    {
        return Err("Supply exactly one supported numeric match predicate.".into());
    }
    Ok(rules[0].clone())
}
