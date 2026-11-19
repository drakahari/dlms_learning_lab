//! Optional ICMP Echo evidence over the existing source-held route/ACL state.
//! No host command, socket, clock, report deserialization or learner callback.
use super::{Draft, Task, binding, core, routing};
use core::echo as model;

pub fn inspect(
    network: &routing::Network,
    packet: &model::Packet,
) -> Result<routing::Inspection, core::Error> {
    network.validate().map_err(|_| {
        core::Error::new(
            core::ErrorKind::Classifier,
            "The supplied virtual network is invalid.",
        )
    })?;
    network
        .inspect(&routing::Flow {
            source: binding(network, packet.source)?,
            destination: binding(network, packet.destination)?,
            protocol: crate::acl::PacketProtocol::Icmp {},
        })
        .map_err(|_| {
            core::Error::new(
                core::ErrorKind::Classifier,
                "This echo path is unsupported; inspect the supplied interfaces and routes.",
            )
        })
}
pub fn reports(task: &Task, draft: &Draft) -> Result<Vec<model::Report>, String> {
    let (_, network) = draft.compile(task)?;
    with_network(task, &network)
}
pub(super) fn with_network(
    task: &Task,
    network: &routing::Network,
) -> Result<Vec<model::Report>, String> {
    model::validate(&task.transport.hosts, &task.echoes).map_err(|e| e.to_string())?;
    task.echoes.iter().map(|query| {
        model::run(&task.transport.hosts, query, &mut |packet| {
            let observed = inspect(network, packet)?;
            if !matches!(observed.baseline, routing::Baseline::Deliverable(_)) {
                return Err(core::Error::new(core::ErrorKind::Classifier, "The supplied route cannot reach the intended host even without policy. No echo reply is assumed."));
            }
            match observed.actual {
                routing::Outcome::Delivered { .. } => Ok(core::Decision::delivered("The supplied virtual path delivered this ICMP packet.")),
                routing::Outcome::Denied { denial, .. } => Ok(core::Decision::dropped(&format!("ICMP filtered at {}/{} {:?}.", denial.at.device, denial.at.interface, denial.at.direction))),
                routing::Outcome::Failed { .. } => Err(core::Error::new(core::ErrorKind::Classifier, "Virtual forwarding failed; no echo receipt is assumed.")),
            }
        }).map_err(|e| e.to_string())
    }).collect()
}
pub(super) fn check(query: &model::Query, report: &model::Report) -> (bool, String) {
    let pass = report.round_trip() && report.received.as_deref() == Some(query.payload.as_slice());
    let why = match report.phase {
        model::Phase::Ready => "No echo request has run.",
        model::Phase::ReplyReady => {
            "The request arrived, but its reply has not returned. Request receipt alone is not a round trip."
        }
        model::Phase::RequestDropped => {
            "The echo request was filtered. Inspect its outward policy trace; no reply can start from this request."
        }
        model::Phase::ReplyDropped => {
            "The echo request arrived, but the reply was filtered on its return path. Inspect the reversed addresses and policy direction."
        }
        model::Phase::Complete if pass => {
            "The matching echo reply returned with the supplied identifier, sequence and unchanged bytes. ICMP has no TCP/UDP service port; this does not prove that an application listener works."
        }
        model::Phase::Complete => {
            "The returned bytes do not match the supplied query. This is not a valid reflected reply."
        }
    };
    (pass, format!("{} — {why}", query.id))
}
