//! Exact typed filter-policy comparison over explicit header/interface domains.
//! This does not prove route, connection/NAT or application behavior.
use crate::linux_filter::{Class, Fault, Verdict};
use crate::linux_filter_regions::{
    Budget, DIMENSIONS, Interval, MAX_STEPS, Rect, RuleSpace, bounded_compare,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mismatch {
    pub packet: crate::linux_filter::Packet,
    pub actual: Verdict,
    pub expected: Verdict,
    pub rule: Option<usize>,
}
fn context(message: &str) -> Fault {
    Fault {
        class: Class::Context,
        message: message.into(),
    }
}
fn cidr_interval(c: Option<crate::linux_filter::Cidr>) -> Interval {
    c.map_or(
        Interval {
            lo: 0,
            hi: u32::MAX,
        },
        |c| {
            let lo = u32::from(c.network);
            let mask = if c.prefix == 0 {
                0
            } else {
                u32::MAX << (32 - c.prefix)
            };
            Interval { lo, hi: lo | !mask }
        },
    )
}
fn predicate(r: &crate::linux_filter::Rule, names: &[String]) -> Option<Rect> {
    use crate::linux_filter::Protocol;
    let mut v = [Interval { lo: 0, hi: 0 }; DIMENSIONS];
    v[0] = cidr_interval(r.source);
    v[1] = cidr_interval(r.destination);
    v[2] = r.protocol.map_or(Interval { lo: 0, hi: 2 }, |p| {
        let n = match p {
            Protocol::Tcp => 0,
            Protocol::Udp => 1,
            Protocol::Icmp => 2,
        };
        Interval { lo: n, hi: n }
    });
    for (d, value) in [(3, r.source_port), (4, r.destination_port)] {
        v[d] = value.map_or(
            Interval {
                lo: 0,
                hi: u16::MAX.into(),
            },
            |n| Interval {
                lo: n.into(),
                hi: n.into(),
            },
        );
    }
    for (d, name) in [(5, &r.ingress), (6, &r.egress)] {
        v[d] = if let Some(name) = name {
            let n = names.iter().position(|n| n == name)? as u32 + 1;
            Interval { lo: n, hi: n }
        } else {
            Interval {
                lo: 0,
                hi: names.len() as u32,
            }
        };
    }
    Some(Rect(v))
}
fn decode(
    point: [u32; DIMENSIONS],
    names: &[String],
) -> Result<crate::linux_filter::Packet, Fault> {
    use crate::linux_filter::{Packet, Protocol};
    let protocol = match point[2] {
        0 => Protocol::Tcp,
        1 => Protocol::Udp,
        2 => Protocol::Icmp,
        _ => return Err(context("Invalid protocol in policy witness.")),
    };
    let device = |n: u32| -> Result<Option<String>, Fault> {
        if n == 0 {
            Ok(None)
        } else {
            names
                .get((n - 1) as usize)
                .cloned()
                .map(Some)
                .ok_or_else(|| context("Invalid device in policy witness."))
        }
    };
    let port = |n: u32| -> Result<Option<u16>, Fault> {
        if protocol == Protocol::Icmp {
            if n != 0 {
                return Err(context("Invalid ICMP port in policy witness."));
            }
            Ok(None)
        } else {
            u16::try_from(n)
                .map(Some)
                .map_err(|_| context("Invalid port in policy witness."))
        }
    };
    Ok(Packet {
        source: point[0].into(),
        destination: point[1].into(),
        protocol,
        source_port: port(point[3])?,
        destination_port: port(point[4])?,
        ingress: device(point[5])?,
        egress: device(point[6])?,
    })
}
/// Compare all supported headers in explicit domains over the supplied interfaces.
/// The allowed predicates describe requirements, not one required command sequence.
/// This proves FILTER behavior only; routes, forwarding, connection tracking and
/// application delivery need their own real model and cannot be inferred here.
pub fn compare_filter(
    table: &crate::linux_filter::Table,
    chain: crate::linux_filter::Chain,
    facts: &crate::linux_router_path::Facts,
    domains: &[crate::linux_filter::Rule],
    allowed: &[crate::linux_filter::Rule],
) -> Result<Option<Mismatch>, Fault> {
    use crate::linux_filter::Chain;
    facts.validate()?;
    table.validate()?;
    if domains.is_empty() || domains.len() > 16 || allowed.len() > 32 {
        return Err(context(
            "Policy comparison needs1to16domains and at most32allowed predicates.",
        ));
    }
    let names = facts
        .links
        .iter()
        .map(|l| l.name.clone())
        .collect::<Vec<_>>();
    let as_requirement = |r: &crate::linux_filter::Rule| -> Result<Rect, Fault> {
        if r.target != Verdict::Accept || !r.valid(chain) {
            return Err(context(
                "Requirements use valid ACCEPT predicates for this chain; they are not executable rules.",
            ));
        }
        predicate(r, &names)
            .ok_or_else(|| context("A policy requirement names an unknown supplied interface."))
    };
    let domains = domains
        .iter()
        .map(as_requirement)
        .collect::<Result<Vec<_>, _>>()?;
    let allowed_regions = allowed
        .iter()
        .map(as_requirement)
        .collect::<Result<Vec<_>, _>>()?;
    let c = table.chain(chain);
    let rules = c
        .rules
        .iter()
        .map(|r| RuleSpace {
            region: predicate(r, &names),
            target: r.target,
        })
        .collect::<Vec<_>>();
    let mut budget = Budget { steps: MAX_STEPS };
    for protocol in 0..=2 {
        let mut world = [Interval {
            lo: 0,
            hi: u32::MAX,
        }; DIMENSIONS];
        world[2] = Interval {
            lo: protocol,
            hi: protocol,
        };
        world[3] = Interval {
            lo: 0,
            hi: if protocol == 2 { 0 } else { u16::MAX.into() },
        };
        world[4] = world[3];
        world[5] = if chain == Chain::Output {
            Interval { lo: 0, hi: 0 }
        } else {
            Interval {
                lo: 1,
                hi: names.len() as u32,
            }
        };
        world[6] = if chain == Chain::Input {
            Interval { lo: 0, hi: 0 }
        } else {
            Interval {
                lo: 1,
                hi: names.len() as u32,
            }
        };
        let mut restricted = vec![];
        for d in &domains {
            if let Some(d) = d.intersect(Rect(world), &mut budget)? {
                restricted.push(d);
            }
        }
        if restricted.is_empty() {
            continue;
        }
        if let Some(counter) =
            bounded_compare(&restricted, &allowed_regions, &rules, c.policy, &mut budget)?
        {
            let packet = decode(counter.point, &names)?;
            let actual = table.evaluate(chain, &packet)?;
            let expected = if allowed.iter().any(|r| r.matches(&packet)) {
                Verdict::Accept
            } else {
                Verdict::Drop
            };
            if actual.verdict != counter.actual
                || actual.rule != counter.rule
                || expected != counter.expected
            {
                return Err(context(
                    "Policy witness disagreed with the typed matcher; no result was accepted.",
                ));
            }
            return Ok(Some(Mismatch {
                packet,
                actual: counter.actual,
                expected: counter.expected,
                rule: counter.rule,
            }));
        }
    }
    Ok(None)
}
