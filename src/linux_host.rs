//! Bounded IPv4 host facts, main-table routes and virtual resolver configuration.
//! No host file/process/network I/O. This is not a kernel or a full iproute2/NSS.
pub mod ipv6_host;
use crate::ipv4_routing::{Assignment, Prefix};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};
pub const HELP: &str = "Simulated administrator command subset (no sudo needed): ip link show; ip link set dev NAME up|down; ip -4 address show [dev NAME]; ip -4 address add|del ADDRESS/PREFIX dev NAME; ip -4 address flush dev NAME; ip -4 route show; ip -4 route add|replace|del PREFIX via GATEWAY dev NAME [metric NUMBER]. getent ahostsv4 NAME reads supplied .lab.test A records through configured IPv4 nameservers. One primary IPv4 address per supplied interface; remove it before adding another. Delete gateway routes before deleting their interface address. Host addresses use conventional usable /1–/30 private/documentation addresses; route prefixes include /0 and /32. Gateway routes require an up configured interface and an on-link gateway. Ambiguous equal-prefix/equal-metric and connected-route collisions are unsupported. No ip address replace, policy tables, IPv6, multipath, namespaces, DHCP, NetworkManager, boot persistence, shell operators or host commands. Native Test address uses supplied path facts, not real packets/RTTs. A virtual resolver editor accepts up to two IPv4 nameserver lines and comments only; no full NSS/search/cache/transport.";
const MAX_RAW: usize = 65536;
fn text<'de, D: serde::Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    crate::cron::text::<D, N>(d)
}
fn sequence<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    crate::cron_workspace::sequence::<D, T, N>(d)
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 15
        && !matches!(s, "." | "..")
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}
fn private(ip: Ipv4Addr) -> bool {
    crate::ipv4_routing::supported_address(ip)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    #[serde(deserialize_with = "text::<_,15>")]
    pub name: String,
    pub network: Prefix,
    pub carrier: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gateway {
    pub address: Ipv4Addr,
    #[serde(deserialize_with = "text::<_,15>")]
    pub device: String,
    pub available: bool,
    #[serde(deserialize_with = "sequence::<_,Prefix,4>")]
    pub exports: Vec<Prefix>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub address: Ipv4Addr,
    pub available: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    #[serde(deserialize_with = "text::<_,80>")]
    pub name: String,
    pub address: Ipv4Addr,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resolver {
    pub address: Ipv4Addr,
    #[serde(deserialize_with = "sequence::<_,Record,8>")]
    pub records: Vec<Record>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Source-only optional facts. Old serialized profiles stay byte-equivalent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<ipv6_host::Facts>,
    #[serde(deserialize_with = "text::<_,48>")]
    pub id: String,
    #[serde(deserialize_with = "sequence::<_,Link,2>")]
    pub links: Vec<Link>,
    #[serde(deserialize_with = "sequence::<_,Gateway,4>")]
    pub gateways: Vec<Gateway>,
    #[serde(deserialize_with = "sequence::<_,Node,8>")]
    pub nodes: Vec<Node>,
    #[serde(deserialize_with = "sequence::<_,Resolver,2>")]
    pub resolvers: Vec<Resolver>,
}
impl Profile {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty()
            || self.id.len() > 48
            || self.links.is_empty()
            || self.links.len() > 2
            || self.gateways.len() > 4
            || self.nodes.len() > 8
            || self.resolvers.len() > 2
        {
            return Err("Virtual host profile exceeds its declared bounds".into());
        }
        let mut names = BTreeSet::new();
        for link in &self.links {
            if !name(&link.name)
                || !names.insert(&link.name)
                || !link.network.valid()
                || !(1..=30).contains(&link.network.length)
                || !private(link.network.address)
            {
                return Err("Invalid supplied interface/network".into());
            }
        }
        if self.links.len() == 2
            && (self.links[0]
                .network
                .contains(self.links[1].network.address)
                || self.links[1]
                    .network
                    .contains(self.links[0].network.address))
        {
            return Err("Overlapping physical LAN facts are outside this subset".into());
        }
        let mut ips = BTreeSet::new();
        for g in &self.gateways {
            let l = self
                .links
                .iter()
                .find(|l| l.name == g.device)
                .ok_or("Unknown gateway interface")?;
            if !(Assignment {
                address: g.address,
                prefix: l.network.length,
            })
            .usable()
                || !l.network.contains(g.address)
                || g.exports.is_empty()
                || g.exports.len() > 4
                || g.exports.iter().any(|p| !p.valid())
                || !ips.insert(g.address)
            {
                return Err("Invalid supplied gateway facts".into());
            }
        }
        for node in &self.nodes {
            if !private(node.address) || !ips.insert(node.address) {
                return Err("Invalid/duplicate supplied host address".into());
            }
        }
        let mut servers = BTreeSet::new();
        for r in &self.resolvers {
            if !servers.insert(r.address)
                || !self.nodes.iter().any(|n| n.address == r.address)
                || r.records.len() > 8
            {
                return Err("Invalid supplied resolver".into());
            }
            let mut records = BTreeSet::new();
            for row in &r.records {
                if !hostname(&row.name)
                    || !records.insert(row.name.to_ascii_lowercase())
                    || !self.nodes.iter().any(|n| n.address == row.address)
                {
                    return Err("Invalid supplied resolver record".into());
                }
            }
        }
        if let Some(facts) = &self.ipv6 { facts.validate(self)?; }
        Ok(())
    }
}
fn hostname(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.to_ascii_lowercase().ends_with(".lab.test")
        && s.split('.').all(|l| {
            !l.is_empty()
                && l.len() <= 63
                && l.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                && !l.starts_with('-')
                && !l.ends_with('-')
        })
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interface {
    #[serde(deserialize_with = "text::<_,15>")]
    pub name: String,
    pub up: bool,
    pub address: Option<Assignment>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub destination: Prefix,
    pub via: Ipv4Addr,
    #[serde(deserialize_with = "text::<_,15>")]
    pub device: String,
    pub metric: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<ipv6_host::Configuration>,
    #[serde(deserialize_with = "sequence::<_,Interface,2>")]
    pub interfaces: Vec<Interface>,
    #[serde(deserialize_with = "sequence::<_,Route,8>")]
    pub routes: Vec<Route>,
    #[serde(deserialize_with = "sequence::<_,Ipv4Addr,2>")]
    pub nameservers: Vec<Ipv4Addr>,
}
impl Configuration {
    pub fn validate(&self, profile: &Profile) -> Result<(), String> {
        profile.validate()?;
        match (&profile.ipv6, &self.ipv6) {
            (None, None) => {},
            (Some(facts), Some(config)) => config.validate(facts, self)?,
            _ => return Err("IPv6 configuration must match the source-enabled host profile".into()),
        }
        if self.interfaces.len() != profile.links.len()
            || self.routes.len() > 8
            || self.nameservers.len() > 2
            || self.nameservers.iter().collect::<BTreeSet<_>>().len() != self.nameservers.len()
        {
            return Err("Invalid virtual host configuration bounds".into());
        }
        if self.nameservers.iter().any(|ip| !private(*ip)) {
            return Err("This activity models only private/documentation IPv4 nameserver addresses; no real server is contacted".into());
        }
        let mut names = BTreeSet::new();
        for i in &self.interfaces {
            if !names.insert(&i.name)
                || !profile.links.iter().any(|l| l.name == i.name)
                || i.address.as_ref().is_some_and(|a| !a.usable())
            {
                return Err("Invalid interface or conventional IPv4 address".into());
            }
        }
        let assigned: Vec<_> = self
            .interfaces
            .iter()
            .filter_map(|i| i.address.as_ref())
            .collect();
        if assigned.len() == 2 {
            let a = Prefix::new(assigned[0].address, assigned[0].prefix).unwrap();
            let b = Prefix::new(assigned[1].address, assigned[1].prefix).unwrap();
            if a.contains(b.address) || b.contains(a.address) {
                return Err("Overlapping connected prefixes are outside this host subset".into());
            }
        }
        let mut keys = BTreeSet::new();
        for r in &self.routes {
            let i = self
                .interfaces
                .iter()
                .find(|i| i.name == r.device)
                .ok_or("Unknown route interface")?;
            if i.address.is_none() {
                return Err(
                    "A modeled gateway route needs a configured primary IPv4 address".into(),
                );
            }
            if !r.destination.valid()
                || !private(r.via)
                || !keys.insert((r.destination.address, r.destination.length, r.metric))
            {
                return Err("Invalid or ambiguous main-table route".into());
            }
            for a in self.interfaces.iter().filter_map(|i| i.address.as_ref()) {
                if r.metric == 0 && r.destination == Prefix::new(a.address, a.prefix).unwrap() {
                    return Err("A route colliding with an automatic connected route is outside this subset".into());
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u32,
    pub profile: Profile,
    pub configuration: Configuration,
    pub generation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub destination: Prefix,
    pub device: String,
    pub source: Ipv4Addr,
    pub via: Option<Ipv4Addr>,
    pub metric: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    pub pass: bool,
    pub reason: String,
    pub selection: Option<Selection>,
}
impl State {
    pub fn new(profile: Profile, configuration: Configuration) -> Result<Self, String> {
        let s = Self {
            version: 1,
            profile,
            configuration,
            generation: 0,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 32768 {
            return Err("Virtual host state version/byte limit".into());
        }
        self.configuration.validate(&self.profile)
    }
    pub fn decode(raw: &[u8], profile: &Profile) -> Result<Self, String> {
        if raw.len() > MAX_RAW {
            return Err("Virtual host data exceeds64KiB".into());
        }
        let s: Self = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
        s.validate()?;
        if s.profile != *profile {
            return Err("Saved network facts differ from the supplied lesson".into());
        }
        Ok(s)
    }
    pub fn same_environment(&self, initial: &Self) -> bool {
        self.validate().is_ok() && initial.validate().is_ok() && self.profile == initial.profile
    }
    fn publish(&mut self, c: Configuration) -> Result<(), String> {
        c.validate(&self.profile)?;
        if c != self.configuration {
            let generation = self
                .generation
                .checked_add(1)
                .ok_or("Virtual host generation limit")?;
            self.configuration = c;
            self.generation = generation;
        }
        Ok(())
    }
    fn interface<'a>(c: &'a mut Configuration, name: &str) -> Result<&'a mut Interface, String> {
        c.interfaces
            .iter_mut()
            .find(|i| i.name == name)
            .ok_or_else(|| format!("Unknown supplied interface {name}"))
    }
    pub fn select(&self, target: Ipv4Addr) -> Result<Selection, String> {
        self.validate()?;
        let mut candidates = vec![];
        for i in &self.configuration.interfaces {
            if let Some(a) = &i.address {
                let p = Prefix::new(a.address, a.prefix).unwrap();
                if p.contains(target) {
                    candidates.push(Selection {
                        destination: p,
                        device: i.name.clone(),
                        source: a.address,
                        via: None,
                        metric: 0,
                    });
                }
            }
        }
        for r in &self.configuration.routes {
            if r.destination.contains(target)
                && let Some(i) = self
                    .configuration
                    .interfaces
                    .iter()
                    .find(|i| i.name == r.device)
                && let Some(a) = &i.address
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
        candidates.sort_by_key(|r| (std::cmp::Reverse(r.destination.length), r.metric));
        let first = candidates
            .first()
            .ok_or("No matching configured IPv4 route")?;
        if candidates.get(1).is_some_and(|r| {
            r.destination.length == first.destination.length && r.metric == first.metric
        }) {
            return Err("Ambiguous equal-prefix/equal-metric lookup is outside this subset".into());
        }
        Ok(first.clone())
    }
    pub fn probe(&self, target: Ipv4Addr) -> Probe {
        let fail = |reason: String, selection: Option<Selection>| Probe {
            pass: false,
            reason,
            selection,
        };
        let selected = match self.select(target) {
            Ok(s) => s,
            Err(e) => return fail(e, None),
        };
        let i = self
            .configuration
            .interfaces
            .iter()
            .find(|i| i.name == selected.device)
            .unwrap();
        let link = self
            .profile
            .links
            .iter()
            .find(|l| l.name == i.name)
            .unwrap();
        let addr = i.address.as_ref().unwrap();
        if !i.up {
            return fail(
                format!("{} is administratively down", i.name),
                Some(selected),
            );
        }
        if !link.carrier {
            return fail(
                format!("{} has no supplied carrier", i.name),
                Some(selected),
            );
        }
        if !link.network.contains(addr.address) {
            return fail(
                format!(
                    "Source address {} is outside {}’s supplied LAN {}",
                    addr.address,
                    i.name,
                    link.network.text()
                ),
                Some(selected),
            );
        }
        if self.profile.nodes.iter().any(|n| n.address == addr.address)
            || self
                .profile
                .gateways
                .iter()
                .any(|g| g.address == addr.address)
        {
            return fail(
                "Source address conflicts with a supplied peer/gateway".into(),
                Some(selected),
            );
        }
        let available = self
            .profile
            .nodes
            .iter()
            .find(|n| n.address == target)
            .map(|n| n.available)
            .or_else(|| {
                self.profile
                    .gateways
                    .iter()
                    .find(|g| g.address == target)
                    .map(|g| g.available)
            });
        if available != Some(true) {
            return fail(
                if available.is_none() {
                    "No supplied virtual destination at this address"
                } else {
                    "The supplied destination is unavailable"
                }
                .into(),
                Some(selected),
            );
        }
        if let Some(via) = selected.via {
            if !Prefix::new(addr.address, addr.prefix)
                .unwrap()
                .contains(via)
            {
                return fail(
                    "Configured gateway is not on-link under the current address/prefix".into(),
                    Some(selected),
                );
            }
            let Some(g) = self
                .profile
                .gateways
                .iter()
                .find(|g| g.address == via && g.device == i.name)
            else {
                return fail(
                    "No supplied gateway at this next hop on the selected LAN".into(),
                    Some(selected),
                );
            };
            if !g.available {
                return fail("The supplied gateway is unavailable".into(), Some(selected));
            }
            if !g.exports.iter().any(|p| p.contains(target)) {
                return fail(
                    "The supplied gateway has no modeled onward path to this destination".into(),
                    Some(selected),
                );
            }
        } else if !link.network.contains(target) {
            return fail("The address mask treats a remote destination as local; it is not on this physical LAN".into(),Some(selected));
        }
        Probe {
            pass: true,
            reason: "Supplied virtual path is usable; no packets or timing were measured".into(),
            selection: Some(selected),
        }
    }
    pub fn resolver_text(&self) -> String {
        self.configuration
            .nameservers
            .iter()
            .map(|ip| format!("nameserver {ip}\n"))
            .collect()
    }
    pub fn save_resolver(&mut self, text: &str) -> Result<(), String> {
        self.validate()?;
        if text.len() > 512 {
            return Err("Virtual resolver text limit is512bytes".into());
        }
        let mut c = self.configuration.clone();
        c.nameservers.clear();
        for line in text.lines() {
            let raw = line.split('#').next().unwrap_or("").trim();
            if raw.is_empty() {
                continue;
            }
            let parts: Vec<_> = raw.split_whitespace().collect();
            let ["nameserver", ip] = parts.as_slice() else {
                return Err("Only IPv4 nameserver lines and # comments are supported here; search/options/NSS directives are not implemented".into());
            };
            let ip = ip
                .parse::<Ipv4Addr>()
                .map_err(|_| "Use a dotted-decimal IPv4 nameserver address")?;
            if c.nameservers.len() == 2 || c.nameservers.contains(&ip) {
                return Err("Use at most two distinct IPv4 nameservers in this activity".into());
            }
            c.nameservers.push(ip);
        }
        self.publish(c)
    }
    pub fn resolve(&self, name: &str) -> Result<Ipv4Addr, String> {
        self.validate()?;
        if !hostname(name) {
            return Err("Use a supplied full .lab.test name; NSS/search/IPv6 behavior is outside this subset".into());
        }
        if self.configuration.nameservers.is_empty() {
            return Err("Localhost-default resolver behavior is outside this activity; configure a supplied IPv4 nameserver".into());
        }
        for server in &self.configuration.nameservers {
            if !self.probe(*server).pass {
                continue;
            }
            let Some(r) = self.profile.resolvers.iter().find(|r| r.address == *server) else {
                continue;
            };
            return r
                .records
                .iter()
                .find(|r| r.name.eq_ignore_ascii_case(name))
                .map(|r| r.address)
                .ok_or(
                    "The first reachable supplied resolver returned no A record for this name"
                        .into(),
                );
        }
        Err("No configured supplied nameserver has a usable virtual path".into())
    }
    pub fn execute(&mut self, input: &str) -> Result<String, String> {
        self.validate()?;
        let help = self.command_help();
        if input.len() > 512
            || !input.is_ascii()
            || input.chars().any(|c| c.is_control() && !matches!(c, '\t'))
            || input.chars().any(|c| ";|&$`<>\\\"'".contains(c))
        {
            return Err(format!(
                "One literal supported command at a time; no shell parsing or host execution. {help}"
            ));
        }
        let parts: Vec<_> = input.split_whitespace().collect();
        let v = parts.as_slice();
        if self.profile.ipv6.is_some() && matches!(v, ["ip", "-6", ..]) {
            return ipv6_host::execute(self, v);
        }
        let mut c = self.configuration.clone();
        match v {
            ["help"] => return Ok(self.command_help().into()),
            ["ip", "link", "show"] => return Ok(self.link_text()),
            ["ip", "link", "set", "dev", dev, state] if matches!(*state, "up" | "down") => {
                Self::interface(&mut c, dev)?.up = *state == "up";
            }
            ["ip", "-4", "address", "show"] => return Ok(self.address_text(None)),
            ["ip", "-4", "address", "show", "dev", dev] => {
                Self::interface(&mut c, dev)?;
                return Ok(self.address_text(Some(dev)));
            }
            ["ip", "-4", "address", op, cidr, "dev", dev] if matches!(*op, "add" | "del") => {
                let address = parse_assignment(cidr)?;
                if *op == "del" && c.routes.iter().any(|r| r.device == *dev) {
                    return Err("Simulator limitation: remove this interface’s gateway routes before deleting its address".into());
                }
                let i = Self::interface(&mut c, dev)?;
                if *op == "add" {
                    if i.address.as_ref() == Some(&address) {
                        return Err("RTNETLINK answers: File exists".into());
                    }
                    if i.address.is_some() {
                        return Err("This activity models one primary IPv4 address per interface. Delete the existing address before adding another; full Linux supports multiple addresses.".into());
                    }
                    i.address = Some(address);
                } else {
                    if i.address.as_ref() != Some(&address) {
                        return Err("That exact IPv4 address/prefix is not configured".into());
                    }
                    i.address = None;
                }
            }
            ["ip", "-4", "address", "flush", "dev", dev] => {
                if c.routes.iter().any(|r| r.device == *dev) {
                    return Err("Simulator limitation: delete this interface’s gateway routes before flushing its address".into());
                }
                Self::interface(&mut c, dev)?.address = None;
            }
            ["ip", "-4", "route", "show"] => return Ok(self.route_text()),
            ["ip", "-4", "route", op, rest @ ..] if matches!(*op, "add" | "replace" | "del") => {
                let r = parse_route(rest)?;
                let existing = c
                    .routes
                    .iter()
                    .position(|x| x.destination == r.destination && x.metric == r.metric);
                if *op == "del" {
                    let at = existing
                        .filter(|n| c.routes[*n] == r)
                        .ok_or("No exact configured gateway route matches those fields")?;
                    c.routes.remove(at);
                } else {
                    let i = Self::interface(&mut c, &r.device)?;
                    let a = i
                        .address
                        .as_ref()
                        .ok_or("Configure an IPv4 address before adding a gateway route")?;
                    if !i.up {
                        return Err(
                            "This activity requires an up interface before adding a gateway route"
                                .into(),
                        );
                    }
                    if !Prefix::new(a.address, a.prefix).unwrap().contains(r.via) {
                        return Err("Nexthop has invalid gateway: it is outside the configured interface prefix".into());
                    }
                    match existing {
                        Some(_) if *op == "add" => {
                            return Err("RTNETLINK answers: File exists".into());
                        }
                        Some(n) => c.routes[n] = r,
                        None => {
                            if c.routes.len() == 8 {
                                return Err("Virtual route-table limit reached".into());
                            }
                            c.routes.push(r);
                        }
                    }
                }
            }
            ["getent", "ahostsv4", name] => {
                let ip = self.resolve(name)?;
                return Ok(format!("{ip} STREAM {name}\n{ip} DGRAM\n{ip} RAW\n"));
            }
            _ => {
                return Err(format!(
                    "Unsupported command/flags in this simulation; nothing ran on the host. {help}"
                ));
            }
        }
        self.publish(c)?;
        Ok(String::new())
    }
    pub fn command_help(&self) -> &'static str {
        if self.profile.ipv6.is_some() { ipv6_host::HELP } else { HELP }
    }
    pub fn select_ipv6(&self, target: std::net::Ipv6Addr) -> Result<ipv6_host::Selection, String> {
        ipv6_host::select(self, target)
    }
    pub fn probe_ipv6(&self, target: std::net::Ipv6Addr) -> ipv6_host::Probe {
        ipv6_host::probe(self, target)
    }
    fn link_text(&self) -> String {
        self.configuration
            .interfaces
            .iter()
            .enumerate()
            .map(|(n, i)| {
                let carrier = self
                    .profile
                    .links
                    .iter()
                    .find(|l| l.name == i.name)
                    .unwrap()
                    .carrier;
                format!(
                    "{}: {}: <{}> state {}\n",
                    n + 2,
                    i.name,
                    if i.up {
                        if carrier {
                            "UP,LOWER_UP"
                        } else {
                            "NO-CARRIER,UP"
                        }
                    } else {
                        ""
                    },
                    if i.up && carrier { "UP" } else { "DOWN" }
                )
            })
            .collect()
    }
    fn address_text(&self, dev: Option<&str>) -> String {
        self.configuration
            .interfaces
            .iter()
            .enumerate()
            .filter(|(_, i)| dev.is_none_or(|d| d == i.name))
            .map(|(n, i)| {
                let mut s = format!(
                    "{}: {}: <{}>\n",
                    n + 2,
                    i.name,
                    if i.up { "UP" } else { "" }
                );
                if let Some(a) = &i.address {
                    s.push_str(&format!(
                        "    inet {}/{} scope global {}\n",
                        a.address, a.prefix, i.name
                    ));
                }
                s
            })
            .collect()
    }
    fn route_text(&self) -> String {
        let mut rows = vec![];
        for i in &self.configuration.interfaces {
            if let Some(a) = &i.address {
                rows.push(format!(
                    "{} dev {} proto kernel scope link src {}{}\n",
                    Prefix::new(a.address, a.prefix).unwrap().text(),
                    i.name,
                    a.address,
                    if i.up { "" } else { " linkdown" }
                ));
            }
        }
        for r in &self.configuration.routes {
            rows.push(format!(
                "{} via {} dev {} metric {}{}\n",
                if r.destination.length == 0 {
                    "default".into()
                } else {
                    r.destination.text()
                },
                r.via,
                r.device,
                r.metric,
                if self
                    .configuration
                    .interfaces
                    .iter()
                    .any(|i| i.name == r.device && i.up)
                {
                    ""
                } else {
                    " linkdown"
                }
            ));
        }
        rows.join("")
    }
}
fn parse_assignment(s: &str) -> Result<Assignment, String> {
    let (ip, prefix) = s
        .split_once('/')
        .ok_or("Use ADDRESS/PREFIX, for example 192.0.2.10/24")?;
    let a = Assignment {
        address: ip
            .parse()
            .map_err(|_| "Malformed dotted-decimal IPv4 address")?,
        prefix: prefix.parse().map_err(|_| "Use a whole-number prefix")?,
    };
    if !a.usable() {
        return Err("This activity supports conventional usable private/documentation IPv4 host addresses with /1–/30; /31,/32 and special addresses are outside its interface subset".into());
    }
    Ok(a)
}
fn parse_route(parts: &[&str]) -> Result<Route, String> {
    let(dest,via,dev,metric)=match parts{[dest,"via",via,"dev",dev]=>(*dest,*via,*dev,0),[dest,"via",via,"dev",dev,"metric",metric]=>(*dest,*via,*dev,metric.parse::<u32>().map_err(|_|"Metric must be a whole number from0 through4294967295")?),_=>return Err("Use PREFIX via GATEWAY dev NAME [metric NUMBER]; other route attributes are not modeled".into())};
    Ok(Route {
        destination: if dest == "default" {
            Prefix::new(Ipv4Addr::UNSPECIFIED, 0).unwrap()
        } else {
            Prefix::parse(dest)?
        },
        via: via.parse().map_err(|_| "Malformed IPv4 gateway")?,
        device: dev.into(),
        metric,
    })
}
