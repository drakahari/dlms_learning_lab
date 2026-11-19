//! Semantic comparisons for supplied settings, not exact source/command matching.
use crate::smtp_config::{Config, Ipv4Prefix};
fn intervals(prefixes: &[Ipv4Prefix]) -> Vec<(u64, u64)> {
    let mut spans = prefixes
        .iter()
        .map(|p| {
            let start = u64::from(u32::from(p.network()));
            (start, start + (1u64 << (32 - p.prefix())) - 1)
        })
        .collect::<Vec<_>>();
    spans.sort_unstable();
    let mut merged: Vec<(u64, u64)> = vec![];
    for (start, end) in spans {
        if let Some(last) = merged.last_mut()
            && start <= last.1 + 1
        {
            last.1 = last.1.max(end);
        } else {
            merged.push((start, end));
        }
    }
    merged
}
pub fn field(a: &Config, b: &Config, key: &str) -> bool {
    match key {
        "myhostname" => a.hostname == b.hostname,
        "mydestination" => a.destinations == b.destinations,
        "relay_domains" => a.relay_domains == b.relay_domains,
        "relayhost" => a.relay == b.relay,
        "mynetworks" => intervals(&a.networks) == intervals(&b.networks),
        "smtpd_relay_restrictions" => [false, true].into_iter().all(|trusted| {
            [false, true].into_iter().all(|authorized| {
                a.policy_decision(trusted, authorized) == b.policy_decision(trusted, authorized)
            })
        }),
        // Parsing already enforces the explicit empty setting in this subset.
        "parent_domain_matches_subdomains" => {
            a.parameters.get(key).is_some_and(String::is_empty)
                && b.parameters.get(key).is_some_and(String::is_empty)
        }
        _ => false,
    }
}
