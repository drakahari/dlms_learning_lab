//! Exact numeric IPv6 filter comparison; not routing, delivery or kernel parity.
use crate::linux_v6_filter::{Chain, Cidr, Class, Fault, Packet, Protocol, Rule, Table, Verdict};
use crate::linux_v6_regions::{
    Budget, DIMENSIONS, Interval, MAX_STEPS, Rect, RuleSpace, bounded_compare,
};
use std::collections::BTreeSet;
const SOURCE: usize = 0;
const DESTINATION: usize = 1;
const INGRESS: usize = 2;
const EGRESS: usize = 3;
const PROTOCOL: usize = 4;
const SPORT: usize = 5;
const DPORT: usize = 6;
const ICMP_TYPE: usize = 7;
const ICMP_CODE: usize = 8;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mismatch {
    pub packet: Packet,
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
fn span(c: Option<Cidr>) -> Interval {
    c.map_or(
        Interval {
            lo: 0,
            hi: u128::MAX,
        },
        |c| {
            let lo = u128::from(c.network);
            let mask = crate::ipv6::mask(c.prefix).expect("Validated IPv6 prefix");
            Interval { lo, hi: lo | !mask }
        },
    )
}
fn exact(n: u128) -> Interval {
    Interval { lo: n, hi: n }
}
fn code(p: Protocol) -> u128 {
    match p {
        Protocol::Tcp => 0,
        Protocol::Udp => 1,
        Protocol::Icmp6 => 2,
    }
}
fn predicate(r: &Rule, names: &[String]) -> Option<Rect> {
    let mut v = [Interval { lo: 0, hi: 0 }; DIMENSIONS];
    v[SOURCE] = span(r.source);
    v[DESTINATION] = span(r.destination);
    v[PROTOCOL] = r
        .protocol
        .map_or(Interval { lo: 0, hi: 2 }, |p| exact(code(p)));
    for (d, value) in [(SPORT, r.source_port), (DPORT, r.destination_port)] {
        v[d] = value.map_or(
            Interval {
                lo: 0,
                hi: u16::MAX.into(),
            },
            |n| exact(n.into()),
        );
    }
    for (d, value) in [(ICMP_TYPE, r.icmp_type), (ICMP_CODE, r.icmp_code)] {
        v[d] = value.map_or(
            Interval {
                lo: 0,
                hi: u8::MAX.into(),
            },
            |n| exact(n.into()),
        );
    }
    for (d, name) in [(INGRESS, &r.ingress), (EGRESS, &r.egress)] {
        v[d] = if let Some(name) = name {
            exact(names.iter().position(|n| n == name)? as u128 + 1)
        } else {
            Interval {
                lo: 0,
                hi: names.len() as u128,
            }
        };
    }
    Some(Rect(v))
}
fn decode(p: [u128; DIMENSIONS], names: &[String]) -> Result<Packet, Fault> {
    let protocol = match p[PROTOCOL] {
        0 => Protocol::Tcp,
        1 => Protocol::Udp,
        2 => Protocol::Icmp6,
        _ => return Err(context("Invalid IPv6 witness protocol.")),
    };
    let dev = |n: u128| -> Result<Option<String>, Fault> {
        if n == 0 {
            Ok(None)
        } else {
            let index =
                usize::try_from(n - 1).map_err(|_| context("Invalid witness device index."))?;
            names
                .get(index)
                .cloned()
                .map(Some)
                .ok_or_else(|| context("Invalid witness device."))
        }
    };
    let port = |n: u128| -> Result<Option<u16>, Fault> {
        if protocol == Protocol::Icmp6 {
            if n != 0 {
                return Err(context("Invalid ICMPv6 witness port."));
            }
            Ok(None)
        } else {
            u16::try_from(n)
                .map(Some)
                .map_err(|_| context("Invalid witness port."))
        }
    };
    let icmp = |n: u128| -> Result<Option<u8>, Fault> {
        if protocol != Protocol::Icmp6 {
            if n != 0 {
                return Err(context("Invalid non-ICMPv6 witness type/code."));
            }
            Ok(None)
        } else {
            u8::try_from(n)
                .map(Some)
                .map_err(|_| context("Invalid witness ICMPv6 type/code."))
        }
    };
    Ok(Packet {
        source: p[SOURCE].into(),
        destination: p[DESTINATION].into(),
        ingress: dev(p[INGRESS])?,
        egress: dev(p[EGRESS])?,
        protocol,
        source_port: port(p[SPORT])?,
        destination_port: port(p[DPORT])?,
        icmp_type: icmp(p[ICMP_TYPE])?,
        icmp_code: icmp(p[ICMP_CODE])?,
    })
}
/// Requirements are ACCEPT predicates over the explicit supplied interfaces.
/// No successful route, connection, neighbor or application is inferred here.
pub fn compare_filter(
    table: &Table,
    chain: Chain,
    names: &[String],
    domains: &[Rule],
    allowed: &[Rule],
) -> Result<Option<Mismatch>, Fault> {
    table.validate()?;
    if names.is_empty()
        || names.len() > 8
        || names.iter().any(|n| {
            n.is_empty()
                || n.len() > 15
                || !n
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b))
        })
        || names.iter().collect::<BTreeSet<_>>().len() != names.len()
    {
        return Err(context("Supply1to8distinct bounded interface names."));
    }
    if domains.is_empty() || domains.len() > 16 || allowed.len() > 32 {
        return Err(context("Use1to16domains and at most32allowed predicates."));
    }
    let requirement = |r: &Rule| -> Result<Rect, Fault> {
        if r.target != Verdict::Accept || !r.valid(chain) {
            return Err(context(
                "Use valid ACCEPT predicates as requirements for this chain.",
            ));
        }
        predicate(r, names)
            .ok_or_else(|| context("A requirement names an unknown supplied interface."))
    };
    let domains = domains
        .iter()
        .map(requirement)
        .collect::<Result<Vec<_>, _>>()?;
    let allowed_regions = allowed
        .iter()
        .map(requirement)
        .collect::<Result<Vec<_>, _>>()?;
    let c = table.chain(chain);
    let rules = c
        .rules
        .iter()
        .map(|r| RuleSpace {
            region: predicate(r, names),
            target: r.target,
        })
        .collect::<Vec<_>>();
    let mut budget = Budget { steps: MAX_STEPS };
    for protocol in [Protocol::Tcp, Protocol::Udp, Protocol::Icmp6] {
        let mut world = [Interval {
            lo: 0,
            hi: u128::MAX,
        }; DIMENSIONS];
        world[PROTOCOL] = exact(code(protocol));
        for d in [SPORT, DPORT] {
            world[d] = Interval {
                lo: 0,
                hi: if protocol == Protocol::Icmp6 {
                    0
                } else {
                    u16::MAX.into()
                },
            };
        }
        for d in [ICMP_TYPE, ICMP_CODE] {
            world[d] = Interval {
                lo: 0,
                hi: if protocol == Protocol::Icmp6 {
                    u8::MAX.into()
                } else {
                    0
                },
            };
        }
        world[INGRESS] = if chain == Chain::Output {
            exact(0)
        } else {
            Interval {
                lo: 1,
                hi: names.len() as u128,
            }
        };
        world[EGRESS] = if chain == Chain::Input {
            exact(0)
        } else {
            Interval {
                lo: 1,
                hi: names.len() as u128,
            }
        };
        let mut restricted = vec![];
        for d in &domains {
            if let Some(r) = d.intersect(Rect(world), &mut budget)? {
                restricted.push(r);
            }
        }
        if restricted.is_empty() {
            continue;
        }
        if let Some(c) =
            bounded_compare(&restricted, &allowed_regions, &rules, c.policy, &mut budget)?
        {
            let packet = decode(c.point, names)?;
            let actual = table.evaluate(chain, &packet)?;
            let expected = if allowed.iter().any(|r| r.matches(&packet)) {
                Verdict::Accept
            } else {
                Verdict::Drop
            };
            if actual.verdict != c.actual || actual.rule != c.rule || expected != c.expected {
                return Err(context(
                    "IPv6 witness disagrees with the independent typed matcher; no proof accepted.",
                ));
            }
            return Ok(Some(Mismatch {
                packet,
                actual: c.actual,
                expected: c.expected,
                rule: c.rule,
            }));
        }
    }
    Ok(None)
}
