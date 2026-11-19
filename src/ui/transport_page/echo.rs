//! Opt-in, source-owned ICMP observations beside the existing configuration tools.
use super::*;
use dlms_learning_lab::transport_core::{
    self as protocol,
    echo::{Kind, Phase},
};
pub(super) const SCOPE: &str = "Echo here models IPv4 unicast request/reply types 8/0, code 0, fixed identifier/sequence and reflected bytes. It has no TCP/UDP service ports. The same virtual route/ACL model inspects each direction; no real ping, latency, checksum wire encoding, ICMP errors or ICMPv6. Policy matches ICMP as a protocol, not individual ICMP types.";

pub(super) fn supplied(ui: &mut egui::Ui, task: &work::Task) {
    if task.echoes.is_empty() {
        return;
    }
    ui.add_space(8.);
    ui.strong("Supplied reachability queries · ICMP Echo");
    lesson_presentation::prose(
        ui,
        "These queries use IP addresses and identifiers, without application ports. Check configuration runs their request and reply paths alongside the TCP/UDP conversations.",
    );
    for q in &task.echoes {
        let address = |id: &str| {
            task.transport
                .hosts
                .iter()
                .find(|h| h.id == id)
                .map(|h| h.address.to_string())
                .unwrap_or_else(|| "Unknown supplied endpoint".into())
        };
        mono(
            ui,
            &format!(
                "{}: {} to {} · identifier {} · sequence {} · {} bytes",
                q.id,
                address(&q.source),
                address(&q.destination),
                q.identifier,
                q.sequence,
                q.payload.len()
            ),
        );
    }
}
fn status(phase: Phase) -> &'static str {
    match phase {
        Phase::Ready => "Not run",
        Phase::ReplyReady => "Request arrived · reply pending",
        Phase::Complete => "Round trip completed",
        Phase::RequestDropped => "Request filtered",
        Phase::ReplyDropped => "Reply filtered on return",
    }
}
pub(super) fn feedback(
    ui: &mut egui::Ui,
    task: &work::Task,
    scope: &work::Scope,
    metrics: &mut Metrics,
) {
    if task.echoes.is_empty() || scope.tested.is_none() {
        return;
    }
    match work::echo::reports(task, &scope.draft) {
        Err(error) => {
            lesson_presentation::prose(ui, &format!("Echo observation unavailable · {error}"));
        }
        Ok(reports) => {
            for report in reports {
                let control_key = format!("echo-{}", report.query);
                let shown = controls::collapsing_header(ui,&format!("Actual ICMP Echo trace · {} · {}",report.query,status(report.phase)))
                .id_salt(("echo-trace",&report.query)).show(ui,|ui| {
                    for (i,event) in report.events.iter().enumerate() {
                        let p=&event.packet;
                        lesson_presentation::prose(ui,&format!("{}. {} · type {}/code {} · {}",i+1,
                            if p.kind==Kind::Request {"Echo request"}else{"Echo reply"},p.kind.type_number(),p.kind.code(),
                            if event.decision.disposition==protocol::Disposition::Delivered {"Delivered to host"}else{"Filtered"}));
                        mono(ui,&format!("{} to {} · no service ports",p.source,p.destination));
                        mono(ui,&format!("identifier {} · sequence {} · {} payload bytes",p.identifier,p.sequence,p.payload.len()));
                        lesson_presentation::prose(ui,&event.decision.detail);
                    }
                    if let Some(bytes)=report.received {
                        ui.strong("Actual reflected reply bytes");
                        let preview=bytes.iter().take(32).map(|b|format!("{b:02x}")).collect::<Vec<_>>().join(" ");
                        mono(ui,&format!("{} bytes · hexadecimal preview: {preview}",bytes.len()));
                        if bytes.len()>32 {lesson_presentation::prose(ui,"The preview shows the first 32 bytes; grading compares the complete reflected content.");}
                    } else {
                        lesson_presentation::prose(ui,"No matching echo reply returned. Inspect the failing direction; this observation alone cannot establish whether a separate application works.");
                    }
                });
                record(metrics, control_key, &shown.header_response, ui);
            }
        }
    }
}
