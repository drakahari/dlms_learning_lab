//! Normal supplied incoming TCP/UDP flow events, not raw packets or a TCP stack.
use crate::{
    linux_connections as ct, linux_filter as base, linux_flow_nat as nat,
    linux_router_path as path, linux_state_filter as filter,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub filter: filter::Table,
    pub nat: nat::Table,
    pub connections: ct::Table,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Gate {
    Forwarded,
    IngressDown,
    NoRoute,
    EgressDown,
    ForwardingDisabled,
    RouterLocal,
    FilterDrop,
    MappingRefused,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    pub gate: Gate,
    pub incoming: ct::Tuple,
    pub filter: ct::Tuple,
    pub outgoing: Option<ct::Tuple>,
    pub class: Option<ct::Class>,
    pub dnat: Option<bool>,
    pub direction: Option<ct::Direction>,
    pub connection: Option<u64>,
    pub route: Option<path::Route>,
    pub filter_rule: Option<usize>,
    pub destination_rule: Option<usize>,
    pub source_rule: Option<usize>,
    pub retained_mapping: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub detail: String,
}
impl Trace {
    fn new(incoming: ct::Tuple) -> Self {
        Self {
            gate: Gate::MappingRefused,
            incoming,
            filter: incoming,
            outgoing: None,
            class: None,
            dnat: None,
            direction: None,
            connection: None,
            route: None,
            filter_rule: None,
            destination_rule: None,
            source_rule: None,
            retained_mapping: false,
            detail: String::new(),
        }
    }
}
impl State {
    pub fn validate(&self) -> Result<(), String> {
        self.filter.validate().map_err(|e| e.message)?;
        self.nat.validate()?;
        self.connections.validate()?;
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 256 * 1024 {
            return Err("Normal-flow state exceeds256KiB.".into());
        }
        Ok(())
    }
    /// Structural/unsupported errors are atomic. A real modeled reply observation can
    /// remain tracked when a later route/filter gate blocks that event.
    pub fn event(
        &mut self,
        facts: &path::Facts,
        forwarding: bool,
        ingress: &str,
        incoming: ct::Tuple,
    ) -> Result<Trace, String> {
        self.validate()?;
        facts.validate().map_err(|e| e.message)?;
        if !incoming.valid() {
            return Err("Use a supplied numeric unicast TCP/UDP flow; raw/ICMP/fragment behavior is outside this profile.".into());
        }
        let mut next = self.clone();
        let result = next.evaluate(facts, forwarding, ingress, incoming)?;
        next.validate()?;
        *self = next;
        Ok(result)
    }
    fn evaluate(
        &mut self,
        facts: &path::Facts,
        forwarding: bool,
        ingress: &str,
        incoming: ct::Tuple,
    ) -> Result<Trace, String> {
        let link = facts
            .links
            .iter()
            .find(|l| l.name == ingress)
            .ok_or("Ingress is not one of the supplied interfaces")?;
        let mut trace = Trace::new(incoming);
        if !link.up {
            trace.gate = Gate::IngressDown;
            trace.detail =
                "The incoming interface is down; this event did not reach connection tracking."
                    .into();
            return Ok(trace);
        }
        let observed = self.connections.observe(incoming)?;
        trace.class = Some(observed.class);
        trace.direction = Some(observed.direction);
        trace.connection = observed.id;
        trace.retained_mapping = observed.id.is_some();
        let forward = if observed.id.is_some() {
            observed.filter
        } else {
            let (t, r) = self.nat.destination(incoming, ingress)?;
            trace.destination_rule = r;
            t
        };
        trace.filter = forward;
        let dnat = if observed.id.is_some() {
            observed.dnat
        } else {
            incoming.destination != forward.destination
        };
        trace.dnat = Some(dnat);
        let protocol = match forward.transport {
            ct::Transport::Tcp => base::Protocol::Tcp,
            ct::Transport::Udp => base::Protocol::Udp,
        };
        let header = path::Header {
            source: forward.source.address,
            destination: forward.destination.address,
            protocol,
            source_port: Some(forward.source.port),
            destination_port: Some(forward.destination.port),
        };
        let route = path::Runtime {
            filter: base::Table::new(base::Verdict::Accept),
            forwarding,
        }
        .evaluate(facts, &path::Origin::Incoming(ingress.into()), &header)
        .map_err(|e| e.message)?;
        trace.route = route.route;
        trace.gate = match route.gate {
            path::Gate::IngressDown => Gate::IngressDown,
            path::Gate::NoRoute => Gate::NoRoute,
            path::Gate::EgressDown => Gate::EgressDown,
            path::Gate::ForwardingDisabled => Gate::ForwardingDisabled,
            path::Gate::Allowed => Gate::Forwarded,
            _ => return Err("Unexpected fixed routing gate; no event committed".into()),
        };
        if trace.gate != Gate::Forwarded {
            trace.detail = match trace.gate {
                Gate::NoRoute => {
                    "No supplied route matches the destination after destination translation."
                }
                Gate::EgressDown => "The chosen outgoing interface is down.",
                Gate::ForwardingDisabled => "IPv4 forwarding is off.",
                _ => "This event did not reach the forwarding filter.",
            }
            .into();
            return Ok(trace);
        }
        if route.chain != Some(base::Chain::Forward) {
            trace.gate = Gate::RouterLocal;
            trace.detail="The destination is still router-local, so there is no forwarded server path in this exercise. Inspect the PREROUTING destination rule.".into();
            return Ok(trace);
        }
        let selected = trace
            .route
            .as_ref()
            .ok_or("Forwarding requires an actual route")?;
        let outgoing_link = facts
            .links
            .iter()
            .find(|l| l.name == selected.device)
            .ok_or("The route names no supplied interface")?;
        let packet = base::Packet {
            source: forward.source.address,
            destination: forward.destination.address,
            protocol,
            source_port: Some(forward.source.port),
            destination_port: Some(forward.destination.port),
            ingress: Some(ingress.into()),
            egress: Some(selected.device.clone()),
        };
        let decision = self
            .filter
            .evaluate_context(base::Chain::Forward, &packet, observed.class, dnat)
            .map_err(|e| e.message)?;
        trace.filter_rule = decision.rule;
        if decision.verdict == base::Verdict::Drop {
            trace.gate = Gate::FilterDrop;
            trace.detail=decision.rule.map_or("No rule matched; the FORWARD policy is DROP.".into(),|r|format!("FORWARD rule {r} drops the post-destination-translation tuple in the displayed connection state."));
            return Ok(trace);
        }
        let wire = if observed.id.is_some() {
            observed.outgoing
        } else {
            let (wire, rule, device) = match self.nat.source(
                forward,
                &selected.device,
                outgoing_link.address,
                &self.connections,
            ) {
                Ok(v) => v,
                Err(e) => {
                    trace.gate = Gate::MappingRefused;
                    trace.detail = e;
                    return Ok(trace);
                }
            };
            trace.source_rule = rule;
            match self.connections.confirm(incoming, forward, wire, device) {
                Ok(id) => trace.connection = Some(id),
                Err(e) => {
                    trace.gate = Gate::MappingRefused;
                    trace.detail = e;
                    return Ok(trace);
                }
            }
            wire
        };
        trace.outgoing = Some(wire);
        trace.detail="This modeled route/filter permits the outgoing tuple; that is not proof of a completed TCP handshake or application exchange.".into();
        Ok(trace)
    }
}
