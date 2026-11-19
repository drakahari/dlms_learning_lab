//! Opt-in host-terminal adapter. Source-only facts; no host I/O or clocks.
use crate::{
    ipv4_routing::Prefix,
    linux_host,
    transport_core::{self as net, echo},
};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

pub const HELP: &str = "This lesson additionally supports ping -c 1 ADDRESS (or ping -4 -c 1 ADDRESS) for its supplied numeric IPv4 peers only. It evaluates one finite virtual Echo request/reply, with no host network, DNS, timer, RTT, TTL, checksum, ICMP errors or IPv6. Other ping flags/counts are unsupported. A returned Echo reply does not prove an application service works; no reply alone does not locate the fault.";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Peer {
    pub address: Ipv4Addr,
    pub responds: bool,
    /// Supplied return-path facts, never derived from outward reachability.
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Prefix,4>")]
    pub return_sources: Vec<Prefix>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Peer,8>")]
    pub peers: Vec<Peer>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exercise {
    pub plan: Plan,
    /// Only positive round trips are graded. Silent peers are investigation
    /// controls, never a negative goal that a broken local path could satisfy.
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Ipv4Addr,8>")]
    pub required_replies: Vec<Ipv4Addr>,
}
impl Exercise {
    pub fn validate(&self, world: &linux_host::State) -> Result<(), String> {
        self.plan.validate(world)?;
        if self.required_replies.is_empty() || self.required_replies.len() > 8 {
            return Err("Echo assignments need 1–8 positive round-trip outcomes.".into());
        }
        for (i, target) in self.required_replies.iter().enumerate() {
            if self.required_replies[..i].contains(target)
                || !self
                    .plan
                    .peers
                    .iter()
                    .any(|p| p.address == *target && p.responds && !p.return_sources.is_empty())
            {
                return Err("Required replies must name distinct supplied responding peers with return paths.".into());
            }
        }
        Ok(())
    }
    pub fn checks(&self, world: &linux_host::State) -> Vec<(bool, String)> {
        if let Err(e) = self.validate(world) {
            return vec![(false, e)];
        }
        self.required_replies
            .iter()
            .map(|target| match self.plan.observe(world, *target) {
                Ok(o) => (
                    o.report.as_ref().is_some_and(echo::Report::round_trip),
                    format!("Echo round trip to {target}: {}", o.detail),
                ),
                Err(e) => (false, format!("Echo round trip to {target}: {e}")),
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub initial: Exercise,
    pub example: Exercise,
}
impl Plan {
    pub fn validate(&self, world: &linux_host::State) -> Result<(), String> {
        world.validate()?;
        if self.peers.is_empty() || self.peers.len() > 8 {
            return Err("Echo diagnosis needs 1–8 explicit supplied peers.".into());
        }
        for (i, peer) in self.peers.iter().enumerate() {
            if self.peers[..i].iter().any(|p| p.address == peer.address)
                || !world
                    .profile
                    .nodes
                    .iter()
                    .any(|n| n.address == peer.address)
                    && !world
                        .profile
                        .gateways
                        .iter()
                        .any(|g| g.address == peer.address)
                || peer.return_sources.len() > 4
                || peer.return_sources.iter().any(|p| !p.valid())
                || peer
                    .return_sources
                    .iter()
                    .enumerate()
                    .any(|(j, p)| peer.return_sources[..j].contains(p))
            {
                return Err("Echo peers must be unique supplied hosts/gateways with bounded, valid return-source prefixes.".into());
            }
        }
        Ok(())
    }
    pub fn command_target(input: &str) -> Result<Ipv4Addr, String> {
        if input.len() > 512
            || !input.is_ascii()
            || input.chars().any(|c| c.is_control() && c != '\t')
        {
            return Err(HELP.into());
        }
        let parts: Vec<_> = input.split_whitespace().collect();
        let target = match parts.as_slice() {
            ["ping", "-c", "1", ip] | ["ping", "-4", "-c", "1", ip] => ip,
            _ => return Err(HELP.into()),
        };
        let ip: Ipv4Addr = target.parse().map_err(|_| "Use a supplied numeric dotted-decimal IPv4 address; this ping subset does not resolve names.")?;
        if ip.to_string() != *target {
            return Err("Use the canonical dotted-decimal form of a supplied IPv4 address.".into());
        }
        Ok(ip)
    }
    pub fn observe(
        &self,
        world: &linux_host::State,
        target: Ipv4Addr,
    ) -> Result<Observation, String> {
        self.validate(world)?;
        let peer = self.peers.iter().find(|p| p.address == target).ok_or("This address has no supplied virtual Echo responder facts; no network query was attempted.")?;
        let path = world.probe(target);
        let Some(selected) = path.selection else {
            return Ok(Observation {
                target,
                selection: None,
                report: None,
                detail: format!("No query evaluated: {}.", path.reason),
            });
        };
        if selected.source == target {
            return Err(
                "Self-address Echo is outside this two-endpoint exercise; choose a supplied peer."
                    .into(),
            );
        }
        let hosts = [
            net::Host {
                id: "workstation".into(),
                address: selected.source,
            },
            net::Host {
                id: "peer".into(),
                address: target,
            },
        ];
        let query = echo::Query {
            id: "host-echo".into(),
            source: "workstation".into(),
            destination: "peer".into(),
            identifier: 617,
            sequence: 1,
            payload: b"DLMS virtual Echo\0".to_vec(),
        };
        let mut session = echo::Session::new(&hosts, &query).map_err(|e| e.to_string())?;
        let return_ok = peer
            .return_sources
            .iter()
            .any(|p| p.contains(selected.source));
        let mut classify = |packet: &echo::Packet| {
            Ok(match packet.kind {
                echo::Kind::Request if path.pass => net::Decision::delivered(
                    "The current supplied outward path delivered the virtual query.",
                ),
                echo::Kind::Request => net::Decision::dropped(&path.reason),
                echo::Kind::Reply if return_ok => net::Decision::delivered(
                    "The peer's supplied return path reaches the selected source.",
                ),
                echo::Kind::Reply => net::Decision::dropped(
                    "The supplied peer return path does not reach this selected source.",
                ),
            })
        };
        session
            .apply(echo::Action::SendRequest, &mut classify)
            .map_err(|e| e.to_string())?;
        if session.report().phase == echo::Phase::ReplyReady && peer.responds {
            session
                .apply(echo::Action::SendReply, &mut classify)
                .map_err(|e| e.to_string())?;
        }
        let report = session.into_report();
        let detail = if report.round_trip() {
            "Matching Echo reply returned with unchanged identifier, sequence and bytes. This does not verify an application listener."
        } else {
            "No matching Echo reply returned. This alone does not distinguish a local path problem, filtering, an unavailable peer or a missing return path; compare the other supplied evidence."
        }.into();
        Ok(Observation {
            target,
            selection: Some(selected),
            report: Some(report),
            detail,
        })
    }
}

/// Ephemeral observation, not deserializable progress or completion authority.
#[derive(Debug)]
pub struct Observation {
    pub target: Ipv4Addr,
    pub selection: Option<linux_host::Selection>,
    pub report: Option<echo::Report>,
    pub detail: String,
}
impl Observation {
    pub fn output(&self) -> String {
        let mut out = format!(
            "Simulated ping · {} · one IPv4 Echo query; no timing or real network\n",
            self.target
        );
        if let Some(s) = &self.selection {
            out.push_str(&format!(
                "Selected source {} · device {} · next hop {}\n",
                s.source,
                s.device,
                s.via
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "direct".into())
            ));
        }
        out.push_str(&self.detail);
        if let Some(r) = &self.report {
            out.push_str(&format!(
                "\nVirtual request evaluated: 1; matching replies: {}. No RTT/TTL measured.",
                usize::from(r.round_trip())
            ));
        }
        out
    }
}
