//! An independently held route work order with bounded editable controls.
use super::table::{self, Entry, Origin, Table};
use crate::{
    automation_workshop::workspace::Outcome,
    ipv4_routing::{self, Prefix},
    ospf,
};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub ospf: Box<ospf::State>,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub ingress: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub primary_interface: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub primary_peer: String,
    pub advertisement: ospf::Endpoint,
    pub static_backup: ipv4_routing::Route,
    pub retained_site: ipv4_routing::Route,
    pub retained_default: ipv4_routing::Route,
    pub probes: [Ipv4Addr; 6],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,8>")]
    pub cost: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8>")]
    pub distance: String,
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub mask_query: String,
}

impl Draft {
    pub fn validate(&self) -> Result<()> {
        for (input, limit) in [(&self.cost, 8), (&self.distance, 8), (&self.mask_query, 15)] {
            if input.len() > limit || input.chars().any(char::is_control) {
                return Err("Enter one short value in each route field.".into());
            }
        }
        Ok(())
    }
}

pub struct BranchCost {
    pub peer: String,
    pub local: u16,
    pub total: Option<u32>,
}

pub struct Observed {
    pub active: Table,
    pub withdrawal: Table,
    pub ospf_candidate: Option<ospf::Route>,
}

impl Task {
    pub fn validate(&self) -> Result<()> {
        let topology = self.ospf.topology();
        let ports = &self.ospf.configuration().routers;
        if !self.ospf.valid()
            || !ports
                .get(&self.ingress)
                .is_some_and(|r| r.interfaces.contains_key(&self.primary_interface))
        {
            return Err("Expected one supplied OSPF cost control.".into());
        }
        let service = topology
            .devices
            .get(&self.advertisement.router)
            .and_then(|r| r.interfaces.get(&self.advertisement.interface))
            .and_then(|port| port.address.as_ref())
            .and_then(|address| Prefix::new(address.address, address.prefix));
        let targets = self.probes;
        let prefix = self.static_backup.destination;
        let wide = self.retained_site.destination;
        if service != Some(prefix)
            || self.retained_default.destination.length != 0
            || wide.length >= prefix.length
            || !wide.contains(prefix.address)
            || targets[0] == targets[1]
            || !prefix.contains(targets[0])
            || !prefix.contains(targets[1])
            || prefix.contains(targets[2])
            || !wide.contains(targets[2])
            || wide.contains(targets[5])
        {
            return Err("Give each comparison a meaningful supplied prefix and address.".into());
        }
        let snapshot = self.ospf.snapshot().map_err(|error| error.message)?;
        let correct_peer = snapshot.routers[&self.ingress].links.iter().any(|link| {
            link.local.interface == self.primary_interface
                && link.compatible
                && link
                    .peer
                    .as_ref()
                    .is_some_and(|peer| peer.router == self.primary_peer)
        });
        let passive_service = port_config(&self.ospf, &self.advertisement)?;
        if !correct_peer
            || !passive_service.passive
            || !passive_service.enabled
            || self.static_backup.interface == self.primary_interface
        {
            return Err("Supply a compatible learned exit, distinct static standby, and passive LAN announcement.".into());
        }
        let state = self.observe(&self.start())?;
        if state.ospf_candidate.is_none()
            || !state
                .withdrawal
                .exact(prefix)
                .is_some_and(|entry| entry.origin == Origin::Static)
        {
            return Err(
                "Supply a directly usable backup and independent OSPF advertisement.".into(),
            );
        }
        Ok(())
    }

    pub fn start(&self) -> Draft {
        Draft {
            cost: self.ospf.configuration().routers[&self.ingress].interfaces
                [&self.primary_interface]
                .cost
                .to_string(),
            distance: self.static_backup.distance.to_string(),
            mask_query: String::new(),
        }
    }

    fn configured(&self, draft: &Draft) -> Result<(ospf::State, u8)> {
        draft.validate()?;
        let cost = integer::<u16>(&draft.cost).ok_or("Enter an OSPF cost from 1 to 65535.")?;
        let distance = integer::<u8>(&draft.distance)
            .ok_or("Enter an administrative distance from 1 to 255.")?;
        let mut state = (*self.ospf).clone();
        let endpoint = ospf::Endpoint::new(&self.ingress, &self.primary_interface);
        let mut port = port_config(&state, &endpoint)?;
        port.cost = cost;
        configure(&mut state, &endpoint, port)?;
        Ok((state, distance))
    }

    pub fn branch_costs(&self, draft: &Draft) -> Result<Vec<BranchCost>> {
        let (state, _) = self.configured(draft)?;
        let ports = state.configuration().routers[&self.ingress]
            .interfaces
            .iter()
            .filter(|(_, port)| {
                port.enabled
                    && !port.passive
                    && port.network_type == ospf::NetworkType::PointToPoint
            })
            .collect::<Vec<_>>();
        let view = state.snapshot().map_err(|error| error.message)?;
        let mut results = Vec::new();
        for &(interface, config) in &ports {
            let peer = view.routers[&self.ingress]
                .links
                .iter()
                .find(|link| &link.local.interface == interface)
                .and_then(|link| link.peer.as_ref())
                .map_or_else(|| interface.clone(), |endpoint| endpoint.router.clone());
            // Ask how this branch would behave with the other local P2P
            // exits inactive. Use a scratch model and keep the live table.
            let mut trial = state.clone();
            for &(other, original) in &ports {
                if other != interface {
                    let mut disabled = original.clone();
                    disabled.enabled = false;
                    configure(
                        &mut trial,
                        &ospf::Endpoint::new(&self.ingress, other),
                        disabled,
                    )?;
                }
            }
            let snapshot = trial.snapshot().map_err(|error| error.message)?;
            let selected = snapshot.routers[&self.ingress].route(self.static_backup.destination);
            results.push(BranchCost {
                peer,
                local: config.cost,
                total: selected.map(|route| route.cost),
            });
        }
        Ok(results)
    }

    pub fn observe(&self, draft: &Draft) -> Result<Observed> {
        let (mut state, distance) = self.configured(draft)?;
        let mut static_backup = self.static_backup.clone();
        static_backup.distance = distance;
        let candidates = [
            static_backup,
            self.retained_site.clone(),
            self.retained_default.clone(),
        ];
        let active = table::derive(&state, &self.ingress, &candidates)?;
        let view = state.snapshot().map_err(|error| error.message)?;
        let ospf_candidate = view.routers[&self.ingress]
            .route(self.static_backup.destination)
            .cloned();

        // A separate hypothetical withdrawal leaves all the live rows intact.
        // Disable only the supplied LAN's advertisement, not its interface.
        let mut advertisement = port_config(&state, &self.advertisement)?;
        advertisement.enabled = false;
        configure(&mut state, &self.advertisement, advertisement)?;
        let withdrawal = table::derive(&state, &self.ingress, &candidates)?;
        Ok(Observed {
            active,
            withdrawal,
            ospf_candidate,
        })
    }

    pub fn inspect<'a>(&self, table: &'a Table, text: &str) -> Result<&'a Entry> {
        let mask: Ipv4Addr = text
            .trim()
            .parse()
            .map_err(|_| "Enter the dotted-decimal mask for the service prefix.")?;
        let bits = u32::from(mask);
        let length = bits.leading_ones() as u8;
        if ipv4_routing::mask(length) != Some(bits) {
            return Err("A network mask has contiguous ones followed by zeros. Check the supplied prefix length.".into());
        }
        let network = self.static_backup.destination.address;
        let prefix = Prefix::new(network, length).ok_or("Unknown network mask.")?;
        if prefix.address != network {
            return Err(format!(
                "That mask describes a network starting at {}. Use the mask for the supplied service address and prefix.",
                prefix.address
            ));
        }
        table.exact(prefix).ok_or_else(|| {
            format!(
                "There is no installed entry for {}. Check the mask and current table.",
                prefix.text()
            )
        })
    }

    pub fn outcomes(&self, draft: &Draft) -> Result<Vec<Outcome>> {
        let observed = self.observe(draft)?;
        let prefix = self.static_backup.destination;
        let preferred = |entry: &Entry| {
            entry.origin == Origin::Ospf
                && entry.destination == prefix
                && entry.exits.len() == 1
                && entry
                    .exits
                    .first()
                    .is_some_and(|hop| hop.interface == self.primary_interface)
        };
        let backup = |entry: &Entry| {
            entry.origin == Origin::Static
                && entry.destination == prefix
                && entry.exits.len() == 1
                && entry.exits.first().is_some_and(|hop| {
                    hop.interface == self.static_backup.interface
                        && hop.next_hop == Some(self.static_backup.via)
                })
        };
        let installed = observed.active.exact(prefix);
        let source_preference = installed.is_some_and(|entry| entry.origin == Origin::Ospf);
        let two_service_lookups = self.probes[..2]
            .iter()
            .all(|ip| observed.active.lookup(*ip).is_some_and(preferred));
        let wider_lookup = observed.active.lookup(self.probes[2]).is_some_and(|entry| {
            entry.origin == Origin::Static && entry.destination == self.retained_site.destination
        });
        let fallback = self.probes[..2]
            .iter()
            .all(|ip| observed.withdrawal.lookup(*ip).is_some_and(backup));
        let retained_other_routes = [
            (3, Origin::Local),
            (4, Origin::Connected),
            (5, Origin::Static),
        ]
        .iter()
        .all(|(index, kind)| {
            observed
                .active
                .lookup(self.probes[*index])
                .is_some_and(|entry| {
                    entry.origin == *kind && (*index != 5 || entry.destination.length == 0)
                })
        });
        let inspection = self.inspect(&observed.active, &draft.mask_query);
        let exact_row = inspection
            .as_ref()
            .is_ok_and(|entry| entry.destination == prefix);
        let detail = match inspection {
            Ok(entry) if exact_row => format!(
                "The query identifies {} with code {}.",
                prefix.text(),
                entry.origin.code(prefix)
            ),
            Ok(entry) => format!(
                "The query returned {}. Inspect the narrower service network without changing the wider route.",
                entry.destination.text()
            ),
            Err(message) => message,
        };
        Ok(vec![
            row(
                "Install OSPF to the exact service prefix",
                source_preference,
                "The live row uses the OSPF source.",
                "Keep the backup's administrative distance higher than OSPF's 110 for this exact prefix.",
            ),
            row(
                "Prefer the requested OSPF exit",
                two_service_lookups,
                "Both inside addresses select only the requested OSPF exit.",
                "Compare outgoing OSPF path costs. Both inside addresses should choose the requested next hop without an equal-cost tie.",
            ),
            row(
                "Keep the wider site route outside the service prefix",
                wider_lookup,
                "The outside address selects the retained wider route.",
                "Check which destinations fall outside the narrower mask.",
            ),
            row(
                "Install the usable backup after withdrawal",
                fallback,
                "Both inside addresses use the static backup when the advertisement is withdrawn.",
                "Keep an eligible static backup for the same prefix. A distance of 255 prevents installation even when OSPF is gone.",
            ),
            row(
                "Preserve local, connected and last-resort entries",
                retained_other_routes,
                "The independent local, connected and default lookups still select their own rows.",
                "Restore the supplied local and broader reachability settings.",
            ),
            Outcome {
                name: "Inspect the service entry with its dotted mask".into(),
                passed: exact_row,
                detail,
            },
        ])
    }
}

fn integer<T: std::str::FromStr + From<u8> + PartialOrd>(text: &str) -> Option<T> {
    let text = text.trim();
    if text.is_empty() || text.len() > 5 || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let value = text.parse().ok()?;
    (value >= 1.into()).then_some(value)
}

fn port_config(state: &ospf::State, endpoint: &ospf::Endpoint) -> Result<ospf::PortConfig> {
    state
        .configuration()
        .routers
        .get(&endpoint.router)
        .and_then(|router| router.interfaces.get(&endpoint.interface))
        .cloned()
        .ok_or("Unknown supplied OSPF interface.".into())
}

fn configure(
    state: &mut ospf::State,
    endpoint: &ospf::Endpoint,
    configuration: ospf::PortConfig,
) -> Result<()> {
    state
        .apply(&ospf::Action::SetPort {
            endpoint: endpoint.clone(),
            configuration,
        })
        .map_err(|error| error.message)
}

fn row(name: &str, passed: bool, success: &str, retry: &str) -> Outcome {
    Outcome {
        name: name.into(),
        passed,
        detail: if passed { success } else { retry }.into(),
    }
}
