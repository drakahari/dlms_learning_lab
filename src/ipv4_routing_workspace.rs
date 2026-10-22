//! Bounded native configuration drafts and pack-only adapters. No host operations.
use crate::ipv4_routing::{Assignment, Prefix, Route, Topology};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::Ipv4Addr};
fn number(text: &str, max: u8) -> Result<u8, String> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("Enter a whole number from 0 to {max}"));
    }
    text.parse::<u8>()
        .ok()
        .filter(|n| *n <= max)
        .ok_or_else(|| format!("Enter a whole number from 0 to {max}"))
}
fn assignment(text: &str) -> Result<Option<Assignment>, String> {
    if text.trim().is_empty() {
        return Ok(None);
    }
    let (address, prefix) = text
        .trim()
        .split_once('/')
        .ok_or("Malformed address/prefix: use an IPv4 address/prefix, such as 192.0.2.10/24")?;
    Ok(Some(Assignment {
        address: address
            .parse()
            .map_err(|_| "Malformed IPv4 address; use four decimal numbers separated by dots")?,
        prefix: {
            let n = number(prefix, 30)?;
            if n == 0 {
                return Err("Interface prefixes use /1–/30; /0 is for routing defaults, and /31-/32 interfaces are outside this model".into());
            }
            n
        },
    }))
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceDraft {
    pub address: String,
    pub up: bool,
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
    #[serde(default)]
    pub show_routes: bool,
    pub selected_device: String,
    pub devices: BTreeMap<String, DeviceDraft>,
}
impl Draft {
    pub fn new(t: &Topology) -> Self {
        Self {
            version: 1,
            show_routes: false,
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
                                            up: i.up,
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
                                    next_hop: r.via.to_string(),
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
                    && d.interfaces
                        .iter()
                        .all(|(id, p)| name(id) && p.address.len() <= 32)
                    && d.routes.iter().all(|r| {
                        r.destination.len() <= 32
                            && r.interface.len() <= 32
                            && r.next_hop.len() <= 32
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
                iface.address =
                    assignment(&draft.address).map_err(|e| format!("{name}/{port}: {e}"))?;
                iface.up = draft.up;
            }
            device.routes = draft
                .routes
                .iter()
                .map(|r| {
                    Ok(Route {
                        destination: Prefix::parse(r.destination.trim())?,
                        interface: r.interface.trim().into(),
                        via: r.next_hop.trim().parse::<Ipv4Addr>().map_err(|_|"Enter an immediate next-hop IPv4 address; blank/interface-only and recursive routes are unsupported")?,
                        distance: number(r.distance.trim(), 255)?,
                    })
                })
                .collect::<Result<_, String>>()
                .map_err(|e| format!("{name}: {e}"))?;
        }
        if !candidate.valid() {
            return Err("Unsupported or invalid configuration: keep known interfaces, nonzero route distances, private/documentation IPv4 addresses and interface prefixes /1–/30.".into());
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
        return Err("Invalid or oversized IPv4 reference action".into());
    }
    let args: Vec<_> = input.split_whitespace().collect();
    let mut draft = Draft::new(t);

    match args.as_slice() {
        ["address",id,port,value]=>{let iface=device(&mut draft,id)?.interfaces.get_mut(*port).ok_or("Unknown interface")?;iface.address=if *value=="none"{String::new()}else{value.to_string()};},
        ["link",id,port,value]=>device(&mut draft,id)?.interfaces.get_mut(*port).ok_or("Unknown interface")?.up=match *value{"up"=>true,"down"=>false,_=>return Err("Use up or down".into())},
        ["forward",id,value]=>device(&mut draft,id)?.forwarding=match *value{"on"=>true,"off"=>false,_=>return Err("Use on or off".into())},
        ["route",id,network,port,via,distance]=>{device(&mut draft,id)?.routes.push(RouteDraft{destination:network.to_string(),interface:port.to_string(),next_hop:via.to_string(),distance:distance.to_string()});},
        ["remove-route",id,index]=>{let index=number(index,31)? as usize;let routes=&mut device(&mut draft,id)?.routes;if index>=routes.len(){return Err("No route at this zero-based row index".into());}routes.remove(index);},
        _=>return Err("Unsupported pack reference action. These adapters are not router/Linux commands; use native configuration editors.".into()),
    }
    draft.apply(t)?;
    Ok("Applied to the simulated IPv4 topology only".into())
}
