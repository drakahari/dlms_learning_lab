//! A local teaching view composed from the existing OSPF and IPv4 models.
use crate::{
    ipv4_routing::{self, Prefix, RouteStatus},
    ospf,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Local,
    Connected,
    Static,
    Ospf,
}

impl Origin {
    pub fn code(self, prefix: Prefix) -> &'static str {
        match self {
            Self::Local => "L",
            Self::Connected => "C",
            Self::Static if prefix.length == 0 => "S*",
            Self::Static => "S",
            Self::Ospf => "O",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Local => "Local interface address",
            Self::Connected => "Connected network",
            Self::Static => "Static",
            Self::Ospf => "OSPF intra-area",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Egress {
    pub interface: String,
    pub next_hop: Option<Ipv4Addr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub destination: Prefix,
    pub origin: Origin,
    pub distance: u8,
    pub metric: u32,
    pub exits: BTreeSet<Egress>,
}

impl Entry {
    pub fn dotted_mask(&self) -> Ipv4Addr {
        ipv4_routing::mask(self.destination.length)
            .expect("Installed prefix length")
            .into()
    }
}

type Key = (Ipv4Addr, u8);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    installed: BTreeMap<Key, Entry>,
    excluded: BTreeMap<Key, Vec<Entry>>,
    pub static_eligibility: Vec<RouteStatus>,
}

impl Table {
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.installed.values()
    }

    pub fn exact(&self, prefix: Prefix) -> Option<&Entry> {
        self.installed.get(&key(prefix))
    }

    pub fn uninstalled(&self, prefix: Prefix) -> &[Entry] {
        self.excluded.get(&key(prefix)).map_or(&[], Vec::as_slice)
    }

    pub fn lookup(&self, address: Ipv4Addr) -> Option<&Entry> {
        self.entries()
            .filter(|entry| entry.destination.contains(address))
            .max_by_key(|entry| entry.destination.length)
    }
}

fn key(prefix: Prefix) -> Key {
    (prefix.address, prefix.length)
}

/// Compose candidates only for one ingress. Determine the best OSPF candidate
/// within the OSPF model before comparing the distance of different sources.
pub fn derive(
    state: &ospf::State,
    ingress: &str,
    static_routes: &[ipv4_routing::Route],
) -> Result<Table, String> {
    if static_routes.len() > 8 {
        return Err("At most eight supplied static candidates can be compared.".into());
    }
    let snapshot = state.snapshot().map_err(|err| err.message)?;
    let view = snapshot
        .routers
        .get(ingress)
        .ok_or("Unknown ingress router.")?;
    let projected = ipv4_routing::Topology {
        devices: BTreeMap::from([(
            ingress.to_owned(),
            ipv4_routing::Device {
                forwarding: true,
                interfaces: state.topology().devices[ingress]
                    .interfaces
                    .iter()
                    .map(|(name, port)| {
                        (
                            name.clone(),
                            ipv4_routing::Interface {
                                link: port.link.clone(),
                                up: port.up,
                                address: port.address.clone(),
                            },
                        )
                    })
                    .collect(),
                routes: static_routes.to_vec(),
            },
        )]),
    };
    if !projected.valid() {
        return Err("Use the supplied IPv4 route and next-hop format.".into());
    }
    let mut candidates = BTreeMap::<Key, Vec<Entry>>::new();
    let mut add = |entry: Entry| {
        candidates
            .entry(key(entry.destination))
            .or_default()
            .push(entry)
    };
    for connected in &view.connected {
        add(Entry {
            destination: connected.destination,
            origin: Origin::Connected,
            distance: 0,
            metric: 0,
            exits: connected
                .interfaces
                .iter()
                .map(|interface| Egress {
                    interface: interface.clone(),
                    next_hop: None,
                })
                .collect(),
        });
    }
    for (interface, port) in &projected.devices[ingress].interfaces {
        if port.up
            && let Some(at) = port.address.as_ref().filter(|at| at.usable())
        {
            add(Entry {
                destination: Prefix::new(at.address, 32).expect("IPv4 host prefix"),
                origin: Origin::Local,
                distance: 0,
                metric: 0,
                exits: BTreeSet::from([Egress {
                    interface: interface.clone(),
                    next_hop: None,
                }]),
            });
        }
    }
    for route in &view.routes {
        add(Entry {
            destination: route.destination,
            origin: Origin::Ospf,
            distance: 110,
            metric: route.cost,
            exits: route
                .next_hops
                .iter()
                .map(|hop| Egress {
                    interface: hop.interface.clone(),
                    next_hop: Some(hop.via),
                })
                .collect(),
        });
    }
    let mut static_eligibility = Vec::new();
    for (i, route) in static_routes.iter().enumerate() {
        let status = projected.route_status(ingress, i)?;
        static_eligibility.push(status);
        if status == RouteStatus::Eligible {
            add(Entry {
                destination: route.destination,
                origin: Origin::Static,
                distance: route.distance,
                metric: 0,
                exits: BTreeSet::from([Egress {
                    interface: route.interface.clone(),
                    next_hop: Some(route.via),
                }]),
            });
        }
    }

    let mut installed = BTreeMap::new();
    for (prefix, alternatives) in &mut candidates {
        alternatives.sort_by_key(|entry| entry.distance);
        let mut best = alternatives.remove(0);
        let tied = alternatives.partition_point(|entry| entry.distance == best.distance);
        if alternatives[..tied]
            .iter()
            .any(|entry| entry.origin != best.origin)
        {
            return Err(format!(
                "Equal administrative distances from unlike sources for {} are outside this exercise. Use different distances.",
                best.destination.text()
            ));
        }
        for entry in alternatives.drain(..tied) {
            best.exits.extend(entry.exits);
        }
        installed.insert(*prefix, best);
    }
    Ok(Table {
        installed,
        excluded: candidates,
        static_eligibility,
    })
}
