//! Exact route-selection comparison for the bounded IPv6 administration profile.
//! Requires only crate-private visibility of existing Facts::all_routes/route.
use crate::{linux_v6_filter::Cidr, linux_v6_path::Facts};
use std::{collections::BTreeSet, net::Ipv6Addr};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Difference {
    pub address: Ipv6Addr,
    pub expected: Option<(String, Option<Ipv6Addr>)>,
    pub actual: Option<(String, Option<Ipv6Addr>)>,
}
fn boundaries(c: Cidr, out: &mut BTreeSet<u128>) -> Result<(), String> {
    if !c.valid() {
        return Err("Route comparison needs canonical IPv6 prefixes.".into());
    }
    let mask = crate::ipv6::mask(c.prefix).ok_or("Invalid IPv6 route prefix.")?;
    let first = u128::from(c.network);
    out.insert(first);
    if let Some(next) = (first | !mask).checked_add(1) {
        out.insert(next);
    }
    Ok(())
}
/// Prefix membership and the existing longest-prefix/metric decision cannot change
/// between these cuts. Compare every such region, not only displayed clients.
/// Metrics may differ when the selected device/next-hop remains equivalent.
/// This compares the declared numeric unicast-route profile, not the kernel FIB,
/// local/address policy tables, multicast routing, NDP or actual packet delivery.
pub fn compare(actual: &Facts, expected: &Facts) -> Result<Option<Difference>, String> {
    actual.validate().map_err(|e| e.message)?;
    expected.validate().map_err(|e| e.message)?;
    let mut cuts = BTreeSet::from([0]);
    for facts in [actual, expected] {
        for route in facts.all_routes().map_err(|e| e.message)? {
            boundaries(route.destination, &mut cuts)?;
        }
    }
    if cuts.len() > 161 {
        return Err("Route comparison exceeded the8link/32route profile.".into());
    }
    for n in cuts {
        let address = Ipv6Addr::from(n);
        let selected = |facts: &Facts| {
            facts
                .route(address)
                .map(|r| r.map(|r| (r.device, r.via)))
                .map_err(|e| e.message)
        };
        let a = selected(actual)?;
        let e = selected(expected)?;
        if a != e {
            return Ok(Some(Difference {
                address,
                expected: e,
                actual: a,
            }));
        }
    }
    Ok(None)
}
