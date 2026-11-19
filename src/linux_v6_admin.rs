//! Opt-in IPv6 router configuration, reusing the existing literal host parser.
//! All interfaces, settings and supplied advertisements are in-process values.
use crate::{
    linux_filter, linux_host as host, linux_v6_filter::Cidr, linux_v6_path as path,
    linux_v6_settings as settings,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv6Addr};
pub const RA_METRIC: u32 = 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Advertisement {
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub device: String,
    pub gateway: Ipv6Addr,
    pub router_lifetime: u16,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u8,
    pub host: host::State,
    pub settings: settings::State,
    /// Last supplied positive-lifetime observations; no elapsed clock is modeled.
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Advertisement,4>")]
    pub learned: Vec<Advertisement>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub text: String,
    pub changed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub accepted: bool,
    pub changed: bool,
    pub detail: String,
}
impl State {
    pub fn new(host: host::State) -> Result<Self, String> {
        let facts = host
            .profile
            .ipv6
            .as_ref()
            .ok_or("Supply an IPv6-enabled host profile.")?;
        let interfaces = facts
            .lans
            .iter()
            .map(|l| settings::Interface {
                name: l.device.clone(),
                forwarding: false,
                force_forwarding: false,
                accept_ra: 1,
            })
            .collect();
        let state = Self {
            version: 1,
            host,
            settings: settings::State {
                version: 1,
                global_forwarding: false,
                interfaces,
            },
            learned: vec![],
        };
        state.validate()?;
        Ok(state)
    }
    fn profile(&self) -> Result<&host::ipv6_host::Facts, String> {
        self.host
            .profile
            .ipv6
            .as_ref()
            .ok_or_else(|| "This router needs a supplied IPv6 profile.".into())
    }
    fn configuration(&self) -> Result<&host::ipv6_host::Configuration, String> {
        self.host
            .configuration
            .ipv6
            .as_ref()
            .ok_or_else(|| "The IPv6 configuration is missing.".into())
    }
    fn combined_routes(&self) -> Result<Vec<host::ipv6_host::Route>, String> {
        let mut routes = self.configuration()?.routes.clone();
        routes.extend(self.learned.iter().map(|a| host::ipv6_host::Route {
            destination: crate::ipv6::Prefix {
                address: Ipv6Addr::UNSPECIFIED,
                length: 0,
            },
            via: a.gateway,
            device: a.device.clone(),
            metric: RA_METRIC,
        }));
        Ok(routes)
    }
    fn known_advertisement(&self, a: &Advertisement) -> Result<(), String> {
        let profile = self.profile()?;
        if !profile
            .gateways
            .iter()
            .any(|g| g.device == a.device && g.address == a.gateway && g.available)
            || crate::ipv6::category(a.gateway) != crate::ipv6::Category::LinkLocal
        {
            return Err("This activity observes only a supplied, available link-local router on its named interface; arbitrary RA sources are not modeled.".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.host.validate()?;
        self.settings.validate().map_err(|e| e.message)?;
        if self.version != 1
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 65536
            || self.learned.len() > 4
        {
            return Err(
                "Use a version 1 router state within 64 KiB and four supplied learned routers."
                    .into(),
            );
        }
        let profile = self.profile()?;
        let configured = self.configuration()?;
        if configured.routes.len() + self.learned.len() > 8 {
            return Err(
                "Simulator limit: keep at most eight combined manual and learned gateway routes."
                    .into(),
            );
        }
        let names = profile
            .lans
            .iter()
            .map(|l| l.device.as_str())
            .collect::<BTreeSet<_>>();
        if names
            != self
                .settings
                .interfaces
                .iter()
                .map(|i| i.name.as_str())
                .collect()
            || names
                != self
                    .host
                    .profile
                    .links
                    .iter()
                    .map(|i| i.name.as_str())
                    .collect()
        {
            return Err("Router settings must name exactly the supplied IPv6 interfaces.".into());
        }
        let mut learned = BTreeSet::new();
        for a in &self.learned {
            self.known_advertisement(a)?;
            if a.router_lifetime == 0 || !learned.insert((&a.device, a.gateway)) {
                return Err("Keep distinct positive-lifetime supplied observations; withdrawals remove the learned entry.".into());
            }
            if configured
                .routes
                .iter()
                .any(|r| r.destination.length == 0 && r.metric == RA_METRIC)
            {
                return Err("Simulator limit: manual and learned defaults at the same metric 1024 are not combined. Use a distinct manual metric; no route selection is invented.".into());
            }
        }
        // A temporarily unassigned interface is valid unfinished work, not corrupt
        // progress. It is not ready for the path evaluator until repaired.
        if configured.interfaces.iter().all(|i| i.address.is_some()) {
            self.raw_facts()?
                .validate()
                .map_err(|e| format!("Simulator route-profile limit: {}", e.message))?;
        }
        Ok(())
    }
    fn raw_facts(&self) -> Result<path::Facts, String> {
        let profile = self.profile()?;
        let configured = self.configuration()?;
        let mut links = vec![];
        for l in &profile.lans {
            let address = configured.interfaces.iter().find(|i| i.device == l.device).and_then(|i| i.address)
                .ok_or_else(||format!("Configure the primary IPv6 address on {} before running a transit test. The unfinished draft is retained.",l.device))?;
            let link = self
                .host
                .profile
                .links
                .iter()
                .find(|p| p.name == l.device)
                .ok_or("Unknown supplied link.")?;
            let current = self
                .host
                .configuration
                .interfaces
                .iter()
                .find(|i| i.name == l.device)
                .ok_or("Unknown configured link.")?;
            links.push(path::Link {
                name: l.device.clone(),
                address: address.address,
                link_local: l.link_local,
                prefix: address.prefix,
                up: current.up && link.carrier,
            });
        }
        let routes = self
            .combined_routes()?
            .iter()
            .map(|r| path::Route {
                destination: Cidr {
                    network: r.destination.address,
                    prefix: r.destination.length,
                },
                device: r.device.clone(),
                via: Some(r.via),
                metric: r.metric,
            })
            .collect::<Vec<_>>();
        Ok(path::Facts { links, routes })
    }
    pub fn facts(&self) -> Result<path::Facts, String> {
        self.validate()?;
        let facts = self.raw_facts()?;
        facts.validate().map_err(|e| e.message)?;
        Ok(facts)
    }
    /// Reuse the supplied physical-neighbor/onward/destination checks as well as
    /// numeric route selection. Route/filter permission alone is not delivery.
    pub fn supplied_path(&self, target: Ipv6Addr) -> Result<host::ipv6_host::Probe, String> {
        self.validate()?;
        if let Some(i) = self
            .configuration()?
            .interfaces
            .iter()
            .find(|i| i.address.is_none())
        {
            return Ok(host::ipv6_host::Probe {
                pass: false,
                reason: format!(
                    "Configure the primary IPv6 address on {} before testing the supplied path.",
                    i.device
                ),
                selection: None,
            });
        }
        let mut host = self.host.clone();
        host.configuration
            .ipv6
            .as_mut()
            .ok_or("Missing IPv6 configuration.")?
            .routes = self.combined_routes()?;
        host.validate()?;
        Ok(host.probe_ipv6(target))
    }
    pub fn command(&mut self, line: &str) -> Result<Output, String> {
        self.validate()?;
        if line.len() > 512 {
            return Err("Keep one literal simulated router command within 512 bytes.".into());
        }
        let words = linux_filter::words(line).map_err(|e| e.message)?;
        let args = words.iter().map(String::as_str).collect::<Vec<_>>();
        let mut next = self.clone();
        let text=match args.as_slice() {
            ["sysctl" | "cat", ..] => {
                let result=next.settings.command(line).map_err(|e| e.message)?;
                if result.forwarding_written==Some(true) {
                    // Linux6.17's forwarding handler purges learned default routers
                    // unless their receiving interface retains accept_ra=2.
                    next.learned.retain(|a| next.settings.interfaces.iter().any(|i|i.name==a.device&&i.accept_ra==2));
                }
                result.text
            }
            ["ip","-6","route","show"] => {
                let mut text=next.host.execute(line)?;
                for a in &next.learned {
                    text.push_str(&format!("default via {} dev {} proto ra metric {RA_METRIC} pref medium\n",a.gateway,a.device));
                }
                text
            }
            ["ip","-6","route",op,parts @ ..] if matches!(*op,"add"|"replace"|"del") => {
                let route=host::ipv6_host::route(parts)?;
                if route.destination.length==0 && route.metric==RA_METRIC && !next.learned.is_empty() {
                    return Err("Simulator limit: editing a learned default through route add/replace/del is outside this activity. Use a supplied lifetime-zero withdrawal; manual routes at distinct metrics remain editable.".into());
                }
                next.host.execute(line)?
            }
            ["ip","-6",..] | ["ip","link","show"] => next.host.execute(line)?,
            _ => return Err("This router activity supports the disclosed IPv6 address/route, link and IPv6 settings commands only; no host shell or files are used.".into()),
        };
        next.validate()?;
        let changed = next != *self;
        *self = next;
        Ok(Output { text, changed })
    }
    /// A supplied already-received RA observation, not packet generation, general
    /// RA parsing, SLAAC, NDP, firewall processing or clock/lifetime expiration.
    pub fn observe_ra(&mut self, a: &Advertisement) -> Result<Observation, String> {
        self.validate()?;
        self.known_advertisement(a)?;
        let up = self
            .host
            .configuration
            .interfaces
            .iter()
            .any(|i| i.name == a.device && i.up)
            && self
                .host
                .profile
                .links
                .iter()
                .any(|l| l.name == a.device && l.carrier);
        if !up {
            return Ok(Observation {accepted:false,changed:false,detail:"The supplied interface is down, so this advertisement cannot be received. No learned route changed.".into()});
        }
        let accepted = self.settings.accepts_ra(&a.device).map_err(|e| e.message)?;
        if !accepted {
            return Ok(Observation {
                accepted: false,
                changed: false,
                detail: format!(
                    "The supplied advertisement on {} was ignored by the current accept_ra/host-router settings. No learned route changed.",
                    a.device
                ),
            });
        }
        let mut next = self.clone();
        next.learned
            .retain(|old| old.device != a.device || old.gateway != a.gateway);
        if a.router_lifetime > 0 {
            next.learned.push(a.clone());
        }
        next.validate()?;
        let changed = next != *self;
        *self = next;
        Ok(Observation {
            accepted: true,
            changed,
            detail: if a.router_lifetime == 0 {
                if changed {
                    "Router lifetime zero withdrew this supplied learned default. Static routes were retained.".into()
                } else {
                    "Accepted the withdrawal, but no matching learned default was present. Static routes were retained.".into()
                }
            } else {
                format!(
                    "Observed a positive router lifetime: learned default via {} on {}, metric {RA_METRIC}. This finite observation does not start a timer or assign an IPv6 address.",
                    a.gateway, a.device
                )
            },
        })
    }
}
