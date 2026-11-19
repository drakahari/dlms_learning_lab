//! IPv6 header path and finite TCP/UDP direction observations, never a TCP stack.
use crate::{
    linux_v6_connections as ct, linux_v6_filter as base, linux_v6_path as path,
    linux_v6_state_filter as filter,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub filter: filter::Table,
    pub connections: ct::Table,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    pub path: path::Trace,
    pub observation: Option<ct::Observation>,
    pub connection: Option<u64>,
    #[serde(deserialize_with = "optional_detail")]
    pub tracking_refused: Option<String>,
}
impl State {
    pub fn validate(&self) -> Result<(), String> {
        self.filter.validate().map_err(|e| e.message)?;
        self.connections.validate()?;
        Ok(())
    }
    /// Unsupported/malformed events are atomic. A real observed reply persists even
    /// when a later route or filter blocks it. ICMPv6 is explicitly stateless.
    pub fn event(
        &mut self,
        facts: &path::Facts,
        forwarding: bool,
        ingress: &str,
        header: &path::Header,
    ) -> Result<Trace, String> {
        self.validate()?;
        let mut selected = path::Runtime {
            filter: base::Table::new(base::Verdict::Accept),
            forwarding,
        }
        .evaluate(facts, &path::Origin::Incoming(ingress.into()), header)
        .map_err(|e| e.message)?;
        let tuple = match header.protocol {
            base::Protocol::Icmp6 => None,
            base::Protocol::Tcp | base::Protocol::Udp => {
                let t = ct::Tuple {
                    transport: if header.protocol == base::Protocol::Tcp {
                        ct::Transport::Tcp
                    } else {
                        ct::Transport::Udp
                    },
                    source: ct::Endpoint {
                        address: header.source,
                        port: header.source_port.ok_or("Missing source port")?,
                    },
                    destination: ct::Endpoint {
                        address: header.destination,
                        port: header.destination_port.ok_or("Missing destination port")?,
                    },
                };
                if !t.valid() {
                    return Err("This tracked TCP/UDP profile supports ordinary unicast endpoints only; scoped link-local TCP/UDP state is not approximated.".into());
                }
                Some(t)
            }
        };
        let mut next = self.clone();
        let observation = if selected.gate == path::Gate::IngressDown {
            None
        } else {
            tuple.map(|t| next.connections.observe(t)).transpose()?
        };
        let mut connection = observation.as_ref().and_then(|o| o.id);
        let mut tracking_refused = None;
        if selected.gate == path::Gate::Allowed {
            let chain = selected.chain.ok_or("Allowed IPv6 packet has no chain")?;
            let decision = next
                .filter
                .evaluate(
                    chain,
                    &base::Packet {
                        source: header.source,
                        destination: header.destination,
                        ingress: Some(ingress.into()),
                        egress: selected.route.as_ref().map(|r| r.device.clone()),
                        protocol: header.protocol,
                        source_port: header.source_port,
                        destination_port: header.destination_port,
                        icmp_type: header.icmp_type,
                        icmp_code: header.icmp_code,
                    },
                    observation.as_ref().map(|o| o.class),
                )
                .map_err(|e| e.message)?;
            selected.rule = decision.rule;
            if decision.verdict == base::Verdict::Drop {
                selected.gate = path::Gate::FilterDrop;
            } else if connection.is_none()
                && let Some(t) = tuple
            {
                match next.connections.confirm(t) {
                    Ok(id) => connection = Some(id),
                    Err(e) => {
                        selected.gate = path::Gate::TrackingRefused;
                        tracking_refused = Some(e);
                    }
                }
            }
        }
        next.validate()?;
        *self = next;
        Ok(Trace {
            path: selected,
            observation,
            connection,
            tracking_refused,
        })
    }
}

fn optional_detail<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Detail(#[serde(deserialize_with = "crate::cron::text::<_,4096>")] String);
    Option::<Detail>::deserialize(d).map(|v| v.map(|v| v.0))
}
