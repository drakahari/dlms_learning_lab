//! Bounded native configuration drafts and pack-only adapters. No host operations.
use crate::ipv6::{Assignment, Prefix, Route, Topology};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::Ipv6Addr};
fn number(text: &str, max: u8) -> Result<u8, String> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("Enter a whole number from 0 to{max}"));
    }
    text.parse::<u8>()
        .ok()
        .filter(|n| *n <= max)
        .ok_or_else(|| format!("Enter a whole number from 0 to{max}"))
}
fn assignment(text: &str, on_link: bool) -> Result<Option<Assignment>, String> {
    if text.trim().is_empty() {
        return Ok(None);
    }
    let (address, prefix) = text
        .trim()
        .split_once('/')
        .ok_or("Use an IPv6 address/prefix, such as 2001:db8:10::10/64")?;
    Ok(Some(Assignment {
        address: address
            .parse()
            .map_err(|_| "Malformed IPv6 address; use colon-separated hexadecimal groups")?,
        prefix: number(prefix, 128)?,
        on_link,
    }))
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceDraft {
    pub address: String,
    pub link_local: String,
    pub up: bool,
    pub on_link: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteDraft {
    pub destination: String,
    pub interface: String,
    pub next_hop: String,
    pub distance: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceDraft {
    pub forwarding: bool,
    pub interfaces: BTreeMap<String, InterfaceDraft>,
    pub routes: Vec<RouteDraft>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub version: u32,
    pub selected_device: String,
    pub devices: BTreeMap<String, DeviceDraft>,
}
impl Draft {
    pub fn new(t: &Topology) -> Self {
        Self {
            version: 1,
            selected_device: t.devices.keys().next().cloned().unwrap_or_default(),
            devices: t
                .devices
                .iter()
                .map(|(name, d)| {
                    (
                        name.clone(),
                        DeviceDraft {
                            forwarding: d.forwarding,
                            interfaces: d
                                .interfaces
                                .iter()
                                .map(|(port, i)| {
                                    (
                                        port.clone(),
                                        InterfaceDraft {
                                            address: i
                                                .address
                                                .as_ref()
                                                .map_or(String::new(), |a| {
                                                    format!("{}/{}", a.address, a.prefix)
                                                }),
                                            link_local: i.link_local.to_string(),
                                            up: i.up,
                                            on_link: i.address.as_ref().is_some_and(|a| a.on_link),
                                        },
                                    )
                                })
                                .collect(),
                            routes: d
                                .routes
                                .iter()
                                .map(|r| RouteDraft {
                                    destination: r.destination.text(),
                                    interface: r.interface.clone(),
                                    next_hop: r.via.map_or(String::new(), |a| a.to_string()),
                                    distance: r.distance.to_string(),
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
        }
    }
    pub fn valid(&self) -> bool {
        let name = |s: &str| {
            !s.is_empty()
                && s.len() <= 32
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        };
        self.version == 1
            && name(&self.selected_device)
            && !self.devices.is_empty()
            && self.devices.len() <= 12
            && self.devices.contains_key(&self.selected_device)
            && self.devices.iter().all(|(id, d)| {
                name(id)
                    && !d.interfaces.is_empty()
                    && d.interfaces.len() <= 8
                    && d.routes.len() <= 32
                    && d.interfaces.iter().all(|(id, p)| {
                        name(id) && p.address.len() <= 64 && p.link_local.len() <= 64
                    })
                    && d.routes.iter().all(|r| {
                        r.destination.len() <= 64
                            && r.interface.len() <= 32
                            && r.next_hop.len() <= 64
                            && r.distance.len() <= 3
                    })
            })
    }
    pub fn same_shape(&self, t: &Topology) -> bool {
        self.valid()
            && self.devices.keys().eq(t.devices.keys())
            && self
                .devices
                .iter()
                .all(|(id, d)| d.interfaces.keys().eq(t.devices[id].interfaces.keys()))
    }
    pub fn apply(&self, t: &mut Topology) -> Result<(), String> {
        if !t.valid() || !self.same_shape(t) {
            return Err("Configuration draft does not match this bounded topology".into());
        }
        let mut candidate = t.clone();
        for (name, draft) in &self.devices {
            let device = candidate.devices.get_mut(name).unwrap();
            if !draft.interfaces.keys().eq(device.interfaces.keys()) {
                return Err(format!("{name}: interface names must stay unchanged"));
            }
            device.forwarding = draft.forwarding;
            for (port, draft) in &draft.interfaces {
                let iface = device.interfaces.get_mut(port).unwrap();
                iface.address = assignment(&draft.address, draft.on_link)
                    .map_err(|e| format!("{name}/{port}: {e}"))?;
                iface.link_local = draft
                    .link_local
                    .parse()
                    .map_err(|_| format!("{name}/{port}: malformed link-local address"))?;
                iface.up = draft.up;
            }
            device.routes = draft
                .routes
                .iter()
                .map(|r| {
                    Ok(Route {
                        destination: Prefix::parse(r.destination.trim())?,
                        interface: r.interface.trim().into(),
                        via: if r.next_hop.trim().is_empty() {
                            None
                        } else {
                            Some(
                                r.next_hop
                                    .trim()
                                    .parse::<Ipv6Addr>()
                                    .map_err(|_| "Malformed next-hop IPv6 address")?,
                            )
                        },
                        distance: number(r.distance.trim(), 255)?,
                    })
                })
                .collect::<Result<_, String>>()
                .map_err(|e| format!("{name}: {e}"))?;
        }
        if !candidate.valid() {
            return Err("Unsupported or invalid configuration: keep known interfaces, nonzero route distances, modeled documentation/locally assigned ULA addresses and conventional fe80::/64 link-local addresses.".into());
        }
        *t = candidate;
        Ok(())
    }
    pub fn matches(&self, t: &Topology) -> bool {
        let mut candidate = t.clone();
        self.apply(&mut candidate).is_ok() && candidate == *t
    }
}
fn device<'a>(t: &'a mut Draft, id: &str) -> Result<&'a mut DeviceDraft, String> {
    t.devices
        .get_mut(id)
        .ok_or_else(|| "Unknown simulated device".into())
}
/// Declarative pack reference actions only. Learners use native editors/probes.
pub fn adapter(t: &mut Topology, input: &str) -> Result<String, String> {
    if input.len() > 4096 || !t.valid() {
        return Err("Invalid or oversized IPv6 reference action".into());
    }
    let args: Vec<_> = input.split_whitespace().collect();
    let mut draft = Draft::new(t);

    match args.as_slice() {
        ["address",id,port,value]=>{let iface=device(&mut draft,id)?.interfaces.get_mut(*port).ok_or("Unknown interface")?;iface.address=if *value=="none"{String::new()}else{value.to_string()};iface.on_link=true;},
        ["link",id,port,value]=>device(&mut draft,id)?.interfaces.get_mut(*port).ok_or("Unknown interface")?.up=match *value{"up"=>true,"down"=>false,_=>return Err("Use up or down".into())},
        ["forward",id,value]=>device(&mut draft,id)?.forwarding=match *value{"on"=>true,"off"=>false,_=>return Err("Use on or off".into())},
        ["route",id,network,port,via,distance]=>{device(&mut draft,id)?.routes.push(RouteDraft{destination:network.to_string(),interface:port.to_string(),next_hop:if *via=="-"{String::new()}else{via.to_string()},distance:distance.to_string()});},
        ["remove-route",id,index]=>{let index=number(index,31)? as usize;let routes=&mut device(&mut draft,id)?.routes;if index>=routes.len(){return Err("No route at this zero-based row index".into());}routes.remove(index);},
        _=>return Err("Unsupported pack reference action. These adapters are not router/Linux commands; use native configuration editors.".into()),
    }
    draft.apply(t)?;
    Ok("Applied to the simulated IPv6 topology only".into())
}
