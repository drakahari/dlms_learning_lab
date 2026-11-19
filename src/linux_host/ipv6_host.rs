//! Bounded IPv6 extension to Linux host facts. No sockets, processes, files or Cisco distance.
use super::{Configuration as HostConfiguration, Profile as HostProfile, State as HostState};
use crate::ipv6::{self, Prefix};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv6Addr};
fn text<'de, D: serde::Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    crate::cron::text::<D, N>(d)
}
fn sequence<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    crate::cron_workspace::sequence::<D, T, N>(d)
}
fn unicast(a: Ipv6Addr) -> bool {
    matches!(ipv6::category(a), ipv6::Category::Documentation) || (u128::from(a) >> 120 == 0xfd)
}
fn route_prefix(p: Prefix) -> bool {
    p.valid() && (p.length == 0 || unicast(p.address))
}
fn link_local(a: Ipv6Addr) -> bool {
    matches!(ipv6::category(a), ipv6::Category::LinkLocal)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lan {
    #[serde(deserialize_with = "text::<_,15>")]
    pub device: String,
    pub network: Prefix,
    pub link_local: Ipv6Addr,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gateway {
    #[serde(deserialize_with = "text::<_,15>")]
    pub device: String,
    pub address: Ipv6Addr,
    pub available: bool,
    #[serde(deserialize_with = "sequence::<_,Prefix,4>")]
    pub exports: Vec<Prefix>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub address: Ipv6Addr,
    pub available: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    #[serde(deserialize_with = "sequence::<_,Lan,2>")]
    pub lans: Vec<Lan>,
    #[serde(deserialize_with = "sequence::<_,Gateway,4>")]
    pub gateways: Vec<Gateway>,
    #[serde(deserialize_with = "sequence::<_,Node,8>")]
    pub nodes: Vec<Node>,
}
impl Facts {
    pub(super) fn validate(&self, host: &HostProfile) -> Result<(), String> {
        if self.lans.is_empty()
            || self.lans.len() > 2
            || self.gateways.len() > 4
            || self.nodes.len() > 8
        {
            return Err("IPv6 supplied facts exceed bounded host limits".into());
        }
        let mut devices = BTreeSet::new();
        for l in &self.lans {
            if !devices.insert(&l.device)
                || !host.links.iter().any(|h| h.name == l.device)
                || !l.network.valid()
                || !(48..=126).contains(&l.network.length)
                || !unicast(l.network.address)
                || !link_local(l.link_local)
            {
                return Err("Invalid supplied IPv6 LAN/interface/link-local facts".into());
            }
        }
        if self.lans.len() == 2
            && (self.lans[0].network.contains(self.lans[1].network.address)
                || self.lans[1].network.contains(self.lans[0].network.address))
        {
            return Err("Overlapping supplied IPv6 LANs are outside this subset".into());
        }
        let mut routers = BTreeSet::new();
        for g in &self.gateways {
            let l = self
                .lans
                .iter()
                .find(|l| l.device == g.device)
                .ok_or("Unknown supplied IPv6 gateway interface")?;
            if !(link_local(g.address) || (unicast(g.address) && l.network.contains(g.address)))
                || g.address == l.link_local
                || !routers.insert((&g.device, g.address))
                || g.exports.is_empty()
                || g.exports.len() > 4
                || g.exports.iter().any(|p| !route_prefix(*p))
            {
                return Err("Invalid supplied IPv6 gateway or exported prefix".into());
            }
        }
        let mut nodes = BTreeSet::new();
        for n in &self.nodes {
            if !unicast(n.address)
                || !nodes.insert(n.address)
                || self.gateways.iter().any(|g| g.address == n.address)
            {
                return Err("Invalid/duplicate supplied IPv6 destination".into());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assignment {
    pub address: Ipv6Addr,
    pub prefix: u8,
}
impl Assignment {
    pub fn network(&self) -> Option<Prefix> {
        Prefix::new(self.address, self.prefix)
    }
    pub fn valid(&self) -> bool {
        unicast(self.address)
            && (48..=126).contains(&self.prefix)
            && self.network().is_some_and(|p| p.address != self.address)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interface {
    #[serde(deserialize_with = "text::<_,15>")]
    pub device: String,
    pub address: Option<Assignment>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub destination: Prefix,
    pub via: Ipv6Addr,
    #[serde(deserialize_with = "text::<_,15>")]
    pub device: String,
    pub metric: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "sequence::<_,Interface,2>")]
    pub interfaces: Vec<Interface>,
    #[serde(deserialize_with = "sequence::<_,Route,8>")]
    pub routes: Vec<Route>,
}
impl Configuration {
    pub(super) fn validate(&self, facts: &Facts, host: &HostConfiguration) -> Result<(), String> {
        if self.interfaces.len() != facts.lans.len()
            || self.interfaces.len() > 2
            || self.routes.len() > 8
        {
            return Err("IPv6 configuration exceeds supplied interface/route bounds".into());
        }
        let mut devices = BTreeSet::new();
        let mut networks = vec![];
        for i in &self.interfaces {
            if !devices.insert(&i.device)
                || !facts.lans.iter().any(|l| l.device == i.device)
                || !host.interfaces.iter().any(|h| h.name == i.device)
                || i.address.as_ref().is_some_and(|a| !a.valid())
            {
                return Err("Invalid IPv6 interface/address; only supplied interfaces and documentation/locally-assigned ULA primary addresses /48–/126 are supported".into());
            }
            if let Some(a) = i.address {
                networks.push(a.network().unwrap());
            }
        }
        if networks.len() == 2
            && (networks[0].contains(networks[1].address)
                || networks[1].contains(networks[0].address))
        {
            return Err("Overlapping IPv6 connected prefixes are outside this subset".into());
        }
        let mut keys = BTreeSet::new();
        for r in &self.routes {
            let a = self
                .interfaces
                .iter()
                .find(|i| i.device == r.device)
                .and_then(|i| i.address)
                .ok_or(
                    "IPv6 gateway routes require a configured primary address in this activity",
                )?;
            if !route_prefix(r.destination)
                || r.metric == 0
                || !keys.insert((r.destination.address, r.destination.length, r.metric))
                || !(link_local(r.via) || (unicast(r.via) && a.network().unwrap().contains(r.via)))
            {
                return Err("Invalid IPv6 route; use a scoped link-local or on-link gateway and an explicit positive metric. Equal-prefix/equal-metric routes are outside this subset".into());
            }
            if networks.contains(&r.destination) {
                return Err("Explicit IPv6 routes colliding with a connected prefix are outside this subset".into());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub destination: Prefix,
    pub device: String,
    pub source: Ipv6Addr,
    pub via: Option<Ipv6Addr>,
    pub metric: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    pub pass: bool,
    pub reason: String,
    pub selection: Option<Selection>,
}
fn configured(host: &HostState) -> Result<(&Facts, &Configuration), String> {
    let f =
        host.profile.ipv6.as_ref().ok_or(
            "IPv6 is not enabled by this supplied lesson; existing IPv4 scope is unchanged",
        )?;
    let c = host
        .configuration
        .ipv6
        .as_ref()
        .ok_or("IPv6 configuration missing")?;
    Ok((f, c))
}
pub(super) fn select(host: &HostState, target: Ipv6Addr) -> Result<Selection, String> {
    host.validate()?;
    let (_, c) = configured(host)?;
    if !unicast(target) {
        return Err("This path view accepts only supplied documentation/ULA unicast destinations; link-local target probes and other scopes are outside this subset".into());
    }
    let mut candidates = vec![];
    for i in &c.interfaces {
        if let Some(a) = i.address
            && let Some(p) = a.network()
            && p.contains(target)
        {
            candidates.push(Selection {
                destination: p,
                device: i.device.clone(),
                source: a.address,
                via: None,
                metric: 256,
            });
        }
    }
    for r in &c.routes {
        if r.destination.contains(target)
            && let Some(a) = c
                .interfaces
                .iter()
                .find(|i| i.device == r.device)
                .and_then(|i| i.address)
        {
            candidates.push(Selection {
                destination: r.destination,
                device: r.device.clone(),
                source: a.address,
                via: Some(r.via),
                metric: r.metric,
            });
        }
    }
    candidates.sort_by_key(|s| (std::cmp::Reverse(s.destination.length), s.metric));
    candidates
        .first()
        .cloned()
        .ok_or("No matching configured IPv6 route".into())
}
pub(super) fn probe(host: &HostState, target: Ipv6Addr) -> Probe {
    let fail = |reason: String, selection: Option<Selection>| Probe {
        pass: false,
        reason,
        selection,
    };
    let s = match select(host, target) {
        Ok(s) => s,
        Err(e) => return fail(e, None),
    };
    let (f, c) = configured(host).expect("selection validated supplied IPv6 context");
    let i = host
        .configuration
        .interfaces
        .iter()
        .find(|i| i.name == s.device)
        .unwrap();
    let l = host
        .profile
        .links
        .iter()
        .find(|l| l.name == s.device)
        .unwrap();
    let lan = f.lans.iter().find(|l| l.device == s.device).unwrap();
    let a = c
        .interfaces
        .iter()
        .find(|i| i.device == s.device)
        .and_then(|i| i.address)
        .unwrap();
    if !i.up {
        return fail(
            format!(
                "{} is administratively down for both address families",
                i.name
            ),
            Some(s),
        );
    }
    if !l.carrier {
        return fail(format!("{} has no supplied carrier", i.name), Some(s));
    }
    if !lan.network.contains(a.address) {
        return fail(
            format!(
                "IPv6 source {} is outside the supplied {} LAN {}",
                a.address,
                i.name,
                lan.network.text()
            ),
            Some(s),
        );
    }
    if f.nodes.iter().any(|n| n.address == a.address)
        || f.gateways.iter().any(|g| g.address == a.address)
    {
        return fail(
            "IPv6 source conflicts with a supplied peer/gateway".into(),
            Some(s),
        );
    }
    if !f.nodes.iter().any(|n| n.address == target && n.available) {
        return fail("No available supplied IPv6 destination; this is a finite path model, not a real packet test".into(),Some(s));
    }
    if let Some(via) = s.via {
        let Some(g) = f
            .gateways
            .iter()
            .find(|g| g.device == s.device && g.address == via)
        else {
            return fail(
                format!(
                    "No supplied IPv6 gateway {via} on {}; link-local next hops are scoped to a device",
                    s.device
                ),
                Some(s),
            );
        };
        if !g.available {
            return fail("Selected IPv6 gateway is unavailable; a configured lower metric does not prove health or automatically select another gateway".into(),Some(s));
        }
        if !g.exports.iter().any(|p| p.contains(target)) {
            return fail(
                "Selected IPv6 gateway has no supplied onward path to this destination".into(),
                Some(s),
            );
        }
    } else if !lan.network.contains(target) {
        return fail(
            "Connected IPv6 selection does not reach the supplied physical LAN".into(),
            Some(s),
        );
    }
    Probe {
        pass: true,
        reason: "Supplied IPv6 path is usable; no NDP, packets, timing or reverse-path simulation"
            .into(),
        selection: Some(s),
    }
}

pub const HELP: &str = "Existing IPv4 forms: ip link show; ip link set dev NAME up|down; ip -4 address show [dev NAME]; ip -4 address add|del ADDRESS/PREFIX dev NAME; ip -4 address flush dev NAME; ip -4 route show; ip -4 route add|replace|del PREFIX via GATEWAY dev NAME [metric N]; getent ahostsv4 NAME. Source-enabled IPv6 extension: ip -6 address show [dev NAME]; ip -6 address add|del ADDRESS/PREFIX dev NAME; ip -6 route show; ip -6 route add|replace|del PREFIX via GATEWAY dev NAME metric N. Use an explicit positive u32 metric. Existing disclosed IPv4 commands and shared ip link operations remain. One manually configured documentation/locally-assigned ULA primary address /48–/126 per supplied interface; IPv6 has no broadcast; subnet-router anycast is outside this activity. Supplied link-local addresses and link-local gateway scope are tied to dev. Remove this interface's IPv6 gateway routes before deleting its primary address. No address replace/flush, extra addresses/flags, RA/SLAAC/DHCPv6/DAD/NDP, privacy/temporary addresses, source-policy/tables/ECMP, connected-prefix route collisions, ping/ip route get, real packets or AAAA lookups. The native path test is simulator-only and uses supplied destination/onward facts; it is not a Linux reachability measurement. Existing IPv4 nameserver input does not configure IPv6 or imply AAAA support.";
fn assignment(s: &str) -> Result<Assignment, String> {
    let (ip, prefix) = s
        .split_once('/')
        .ok_or("Use a literal IPv6 address/prefix")?;
    if prefix.is_empty() || !prefix.bytes().all(|b| b.is_ascii_digit()) {
        return Err("IPv6 address prefix must be a whole number /48–/126 in this activity".into());
    }
    let a = Assignment {
        address: ip.parse().map_err(|_| "Malformed IPv6 address")?,
        prefix: prefix.parse().map_err(|_| "Invalid IPv6 address prefix")?,
    };
    if !a.valid() {
        return Err("Unsupported IPv6 address form; use a documentation/locally-assigned ULA primary address /48–/126. The all-zero subnet-router anycast address is outside this activity; IPv6 has no broadcast. Link-local addresses are supplied rather than edited in this activity".into());
    }
    Ok(a)
}
pub(crate) fn route(v: &[&str]) -> Result<Route, String> {
    let [d, "via", gateway, "dev", device, "metric", metric] = v else {
        return Err("Use PREFIX via GATEWAY dev NAME metric N; this IPv6 subset requires an explicit positive metric and one gateway".into());
    };
    if metric.is_empty() || !metric.bytes().all(|b| b.is_ascii_digit()) {
        return Err("IPv6 metric must be a positive whole number".into());
    }
    let r = Route {
        destination: if *d == "default" {
            Prefix::new(Ipv6Addr::UNSPECIFIED, 0).unwrap()
        } else {
            Prefix::parse(d)?
        },
        via: gateway.parse().map_err(|_| "Malformed IPv6 gateway")?,
        device: (*device).into(),
        metric: metric.parse().map_err(|_| "IPv6 metric exceeds u32")?,
    };
    if r.metric == 0 {
        return Err("This finite IPv6 subset requires metric1–4294967295; it does not approximate Linux's zero/default metric normalization".into());
    }
    Ok(r)
}
pub(super) fn execute(host: &mut HostState, v: &[&str]) -> Result<String, String> {
    host.validate()?;
    let (f, c) = configured(host)?;
    match v {
        ["ip", "-6", "address", "show"] | ["ip", "-6", "address", "show", "dev", _] => {
            let dev = v.get(5).copied();
            if dev.is_some_and(|d| !c.interfaces.iter().any(|i| i.device == d)) {
                return Err("Unknown supplied IPv6 interface".into());
            }
            let mut out = String::new();
            for i in c
                .interfaces
                .iter()
                .filter(|i| dev.is_none_or(|d| d == i.device))
            {
                out.push_str(&format!("{}:\n", i.device));
                if let Some(a) = i.address {
                    out.push_str(&format!(
                        "    inet6 {}/{} scope global\n",
                        a.address, a.prefix
                    ));
                }
                if host
                    .configuration
                    .interfaces
                    .iter()
                    .any(|h| h.name == i.device && h.up)
                {
                    let l = f.lans.iter().find(|l| l.device == i.device).unwrap();
                    out.push_str(&format!("    inet6 {}/64 scope link\n", l.link_local));
                }
            }
            return Ok(out);
        }
        ["ip", "-6", "route", "show"] => {
            let mut out = String::new();
            for i in &c.interfaces {
                if let Some(a) = i.address {
                    out.push_str(&format!(
                        "{} dev {} metric 256{}\n",
                        a.network().unwrap().text(),
                        i.device,
                        if host
                            .configuration
                            .interfaces
                            .iter()
                            .any(|h| h.name == i.device && h.up)
                        {
                            ""
                        } else {
                            " linkdown"
                        }
                    ));
                }
            }
            for r in &c.routes {
                out.push_str(&format!(
                    "{} via {} dev {} metric {}{}\n",
                    if r.destination.length == 0 {
                        "default".into()
                    } else {
                        r.destination.text()
                    },
                    r.via,
                    r.device,
                    r.metric,
                    if host
                        .configuration
                        .interfaces
                        .iter()
                        .any(|h| h.name == r.device && h.up)
                    {
                        ""
                    } else {
                        " linkdown"
                    }
                ));
            }
            return Ok(out);
        }
        _ => {}
    }
    let mut configuration = host.configuration.clone();
    let c = configuration.ipv6.as_mut().unwrap();
    match v {
        ["ip", "-6", "address", op, cidr, "dev", dev] if matches!(*op, "add" | "del") => {
            let a = assignment(cidr)?;
            let i = c
                .interfaces
                .iter_mut()
                .find(|i| i.device == *dev)
                .ok_or("Unknown supplied IPv6 interface")?;
            if *op == "add" {
                if i.address.is_some() {
                    return Err("This activity models one manually configured IPv6 primary per interface. Delete the old exact address first; real Linux can hold multiple addresses".into());
                }
                i.address = Some(a);
            } else {
                if i.address != Some(a) {
                    return Err("That exact IPv6 address/prefix is not configured".into());
                }
                if c.routes.iter().any(|r| r.device == *dev) {
                    return Err("Simulator limitation: remove this interface's IPv6 gateway routes before deleting its primary address".into());
                }
                i.address = None;
            }
        }
        ["ip", "-6", "route", op, rest @ ..] if matches!(*op, "add" | "replace" | "del") => {
            let r = route(rest)?;
            let at = c
                .routes
                .iter()
                .position(|x| x.destination == r.destination && x.metric == r.metric);
            if *op == "del" {
                let at = at
                    .filter(|n| c.routes[*n] == r)
                    .ok_or("No exact IPv6 route matches all supplied fields")?;
                c.routes.remove(at);
            } else {
                let a=c.interfaces.iter().find(|i|i.device==r.device).and_then(|i|i.address).ok_or("Configure a primary IPv6 address before adding a gateway route in this activity")?;
                if !host
                    .configuration
                    .interfaces
                    .iter()
                    .any(|i| i.name == r.device && i.up)
                {
                    return Err("This activity requires an up interface before adding an IPv6 gateway route".into());
                }
                if !(link_local(r.via) || (unicast(r.via) && a.network().unwrap().contains(r.via)))
                {
                    return Err("The IPv6 gateway must be link-local scoped by dev or in this interface's configured prefix".into());
                }
                match at {
                    Some(n) if *op == "replace" => c.routes[n] = r,
                    Some(_) => {
                        return Err(
                            "An IPv6 route with that exact prefix/metric already exists".into()
                        );
                    }
                    None => {
                        if c.routes.len() == 8 {
                            return Err("IPv6 route-table limit reached".into());
                        }
                        c.routes.push(r);
                    }
                }
            }
        }
        _ => {
            return Err(format!(
                "Unsupported IPv6 syntax or flags; previous state retained. {HELP}"
            ));
        }
    }
    host.publish(configuration)?;
    Ok(String::new())
}
