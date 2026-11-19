//! Exact first-packet translation properties, specialized per TCP/UDP protocol.
//! Derived numeric rules reuse the existing bounded seven-dimensional proof.
use crate::{
    linux_connections::{Endpoint, Transport},
    linux_filter::{self as base, Chain, Cidr, Class, Fault, Protocol, Rule, Verdict},
    linux_filter_policy::{self as policy, Mismatch},
    linux_flow_nat as nat,
    linux_router_path::Facts,
};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Requirement {
    SourceAddress(Ipv4Addr),
    ChangedToDestination(Endpoint),
    DestinationOrIdentity(Endpoint),
}
fn fault(class: Class, message: &str) -> Fault {
    Fault {
        class,
        message: message.into(),
    }
}
fn exact_ip(r: &Rule, address: Ipv4Addr, source: bool) -> Option<Rule> {
    let old = if source { r.source } else { r.destination };
    if old.is_some_and(|c| !c.contains(address)) {
        return None;
    }
    let mut r = r.clone();
    let value = Some(Cidr {
        network: address,
        prefix: 32,
    });
    if source {
        r.source = value;
    } else {
        r.destination = value;
    }
    Some(r)
}
fn exact_port(r: &Rule, port: u16) -> Option<Rule> {
    if r.destination_port.is_some_and(|p| p != port) {
        return None;
    }
    let mut r = r.clone();
    r.destination_port = Some(port);
    Some(r)
}
fn identity_destination(r: &Rule, address: Ipv4Addr, port: Option<u16>) -> Option<Rule> {
    let r = exact_ip(r, address, false)?;
    if let Some(p) = port {
        exact_port(&r, p)
    } else {
        Some(r)
    }
}
fn wanted_destination(
    r: &Rule,
    address: Ipv4Addr,
    port: Option<u16>,
    wanted: Endpoint,
) -> Option<Rule> {
    if address != wanted.address {
        return None;
    }
    match port {
        Some(p) if p == wanted.port => Some(r.clone()),
        Some(_) => None,
        None => exact_port(r, wanted.port),
    }
}
fn push(rules: &mut Vec<Rule>, r: Option<Rule>, verdict: Verdict) -> Result<(), Fault> {
    if let Some(mut r) = r {
        if rules.len() >= base::MAX_RULES {
            return Err(fault(
                Class::Limit,
                "The exact translation proof exceeds96derived rules; simplify the rule set. No successful result was recorded.",
            ));
        }
        r.target = verdict;
        rules.push(r);
    }
    Ok(())
}
fn unicast(a: Ipv4Addr) -> bool {
    !a.is_unspecified()
        && !a.is_multicast()
        && !a.is_broadcast()
        && !a.is_loopback()
        && !a.is_link_local()
}
pub fn property_table(
    table: &nat::Table,
    facts: &Facts,
    transport: Transport,
    requirement: Requirement,
) -> Result<(base::Table, Chain), Fault> {
    table.validate().map_err(|m| fault(Class::Context, &m))?;
    facts.validate()?;
    let address = match requirement {
        Requirement::SourceAddress(a) => a,
        Requirement::ChangedToDestination(e) | Requirement::DestinationOrIdentity(e) => e.address,
    };
    if !unicast(address) {
        return Err(fault(
            Class::Context,
            "Translation requirements need a supplied numeric unicast endpoint.",
        ));
    }
    let protocol = match transport {
        Transport::Tcp => Protocol::Tcp,
        Transport::Udp => Protocol::Udp,
    };
    let is_source = matches!(requirement, Requirement::SourceAddress(_));
    let chain = if is_source {
        Chain::Output
    } else {
        Chain::Input
    };
    let mut derived = vec![];
    for item in if is_source {
        &table.postrouting
    } else {
        &table.prerouting
    } {
        if item.predicate.protocol.is_some_and(|p| p != protocol) {
            continue;
        }
        let mut r = item.predicate.clone();
        r.protocol = Some(protocol);
        match (requirement, item.target) {
            (Requirement::SourceAddress(wanted), nat::Target::Snat { address, .. }) => {
                push(
                    &mut derived,
                    Some(r),
                    if address == wanted {
                        Verdict::Accept
                    } else {
                        Verdict::Drop
                    },
                )?;
            }
            (Requirement::SourceAddress(wanted), nat::Target::Masquerade { .. }) => {
                let valid = facts
                    .links
                    .iter()
                    .find(|l| l.address == wanted)
                    .and_then(|l| {
                        if r.egress.as_ref().is_some_and(|n| n != &l.name) {
                            None
                        } else {
                            let mut good = r.clone();
                            good.egress = Some(l.name.clone());
                            Some(good)
                        }
                    });
                push(&mut derived, valid, Verdict::Accept)?;
                push(&mut derived, Some(r), Verdict::Drop)?;
            }
            (Requirement::ChangedToDestination(wanted), nat::Target::Dnat { address, port }) => {
                let good = wanted_destination(&r, address, port, wanted);
                // An explicit identity DNAT target does not set destination-translation status.
                let identity = good
                    .as_ref()
                    .and_then(|g| identity_destination(g, wanted.address, Some(wanted.port)));
                push(&mut derived, identity, Verdict::Drop)?;
                push(&mut derived, good, Verdict::Accept)?;
                push(&mut derived, Some(r), Verdict::Drop)?;
            }
            (Requirement::DestinationOrIdentity(wanted), nat::Target::Dnat { address, port }) => {
                push(
                    &mut derived,
                    identity_destination(&r, address, port),
                    Verdict::Accept,
                )?;
                push(
                    &mut derived,
                    wanted_destination(&r, address, port, wanted),
                    Verdict::Accept,
                )?;
                push(&mut derived, Some(r), Verdict::Drop)?;
            }
            _ => {
                return Err(fault(
                    Class::Context,
                    "The translation requirement names the wrong NAT hook.",
                ));
            }
        }
    }
    let default = if matches!(requirement, Requirement::DestinationOrIdentity(_)) {
        Verdict::Accept
    } else {
        Verdict::Drop
    };
    if let Requirement::SourceAddress(address) = requirement {
        // With no source rule, identity satisfies an address property only for that address.
        push(
            &mut derived,
            Some(Rule {
                source: Some(Cidr {
                    network: address,
                    prefix: 32,
                }),
                destination: None,
                ingress: None,
                egress: None,
                protocol: Some(protocol),
                source_port: None,
                destination_port: None,
                target: Verdict::Accept,
            }),
            Verdict::Accept,
        )?;
    }
    let mut result = base::Table::new(default);
    if is_source {
        result.output.rules = derived;
    } else {
        result.input.rules = derived;
    }
    result.validate()?;
    Ok((result, chain))
}
fn witness_property(
    table: &nat::Table,
    facts: &Facts,
    requirement: Requirement,
    p: &base::Packet,
) -> Result<(bool, Option<usize>), Fault> {
    let source = matches!(requirement, Requirement::SourceAddress(_));
    let chain = if source { Chain::Output } else { Chain::Input };
    let rules = if source {
        &table.postrouting
    } else {
        &table.prerouting
    };
    let mut selector = base::Table::new(Verdict::Drop);
    let predicates = rules.iter().map(|r| r.predicate.clone()).collect();
    if source {
        selector.output.rules = predicates;
    } else {
        selector.input.rules = predicates;
    }
    let selected = selector.evaluate(chain, p)?.rule;
    let target = selected.map(|n| rules[n - 1].target);
    let value = match requirement {
        Requirement::SourceAddress(wanted) => {
            let actual = match target {
                None => p.source,
                Some(nat::Target::Snat { address, .. }) => address,
                Some(nat::Target::Masquerade { .. }) => {
                    facts
                        .links
                        .iter()
                        .find(|l| p.egress.as_deref() == Some(l.name.as_str()))
                        .ok_or_else(|| {
                            fault(
                                Class::Context,
                                "NAT witness has no supplied outgoing interface",
                            )
                        })?
                        .address
                }
                _ => {
                    return Err(fault(
                        Class::Context,
                        "Source witness matched a destination target",
                    ));
                }
            };
            actual == wanted
        }
        Requirement::ChangedToDestination(wanted) | Requirement::DestinationOrIdentity(wanted) => {
            let original = Endpoint {
                address: p.destination,
                port: p.destination_port.ok_or_else(|| {
                    fault(
                        Class::Context,
                        "A normal-flow witness needs a destination port",
                    )
                })?,
            };
            let actual = match target {
                None => original,
                Some(nat::Target::Dnat { address, port }) => Endpoint {
                    address,
                    port: port.unwrap_or(original.port),
                },
                _ => {
                    return Err(fault(
                        Class::Context,
                        "Destination witness matched a source target",
                    ));
                }
            };
            if matches!(requirement, Requirement::ChangedToDestination(_)) {
                actual == wanted && actual != original
            } else {
                actual == wanted || actual == original
            }
        }
    };
    Ok((value, selected))
}
/// All domains/expected predicates explicitly bind one protocol. The caller checks
/// both protocols where required; this never treats a sampled client as a proof.
pub fn compare(
    table: &nat::Table,
    facts: &Facts,
    transport: Transport,
    requirement: Requirement,
    domains: &[Rule],
    expected_true: &[Rule],
) -> Result<Option<Mismatch>, Fault> {
    let protocol = match transport {
        Transport::Tcp => Protocol::Tcp,
        Transport::Udp => Protocol::Udp,
    };
    if domains
        .iter()
        .chain(expected_true)
        .any(|r| r.protocol != Some(protocol))
    {
        return Err(fault(
            Class::Context,
            "Declare the same explicit TCP or UDP protocol on every translation-proof predicate.",
        ));
    }
    let (t, chain) = property_table(table, facts, transport, requirement)?;
    let mut result = policy::compare_filter(&t, chain, facts, domains, expected_true)?;
    if let Some(mismatch) = &mut result {
        let (actual, original_rule) =
            witness_property(table, facts, requirement, &mismatch.packet)?;
        if actual != (mismatch.actual == Verdict::Accept) {
            return Err(fault(
                Class::Context,
                "Translation proof disagreed with its actual first-match witness; no grade was accepted.",
            ));
        }
        mismatch.rule = original_rule; // Report the NAT rule, not an internal derived-rule index.
    }
    Ok(result)
}
