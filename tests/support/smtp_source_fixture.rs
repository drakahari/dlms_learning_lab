//! Synthetic source binding; not a registered lesson or content acceptance.
#[path = "smtp_tasks.rs"]
mod tasks;
use dlms_learning_lab::{
    engine::{Goal, State},
    pack, smtp_source,
    smtp_workspace::{Action, Session},
};
pub fn encode(a: &Action) -> String {
    format!(
        "{}{}",
        smtp_source::PREFIX,
        serde_json::to_string(a).unwrap()
    )
}
pub fn scenario() -> pack::Scenario {
    let mut s = pack::load(include_str!(
        "../../scenarios/kernel-module-lessons-v1.json"
    ))
    .unwrap()
    .remove(0);
    let d = tasks::definition();
    s.id = d.source_id.clone();
    s.track = "LPIC-2".into();
    s.unit = "test-smtp-unit".into();
    s.title = "Repair the supplied mail route".into();
    s.objective="Deliver every supplied sample to its intended mailbox and preserve rejection of untrusted external relay attempts. Save and reload the repaired next hop, then retry queued mail.".into();
    s.guide = vec![pack::LessonStep {
        title: "Inspect separate example settings".into(),
        why: "Mail moves between supplied servers. Saving and reloading are different actions."
            .into(),
        action: "Select the guide server and run postconf -n.".into(),
        commands: vec![pack::CommandExample {
            command: "postconf -n".into(),
            explanation: "Read the saved literal settings on the selected simulated server.".into(),
            simulator_only: false,
        }],
    }];
    s.steps = vec!["Understand the task".into()];
    s.initial = State::SmtpRouting {
        simulation: Box::new(Session::new(&d).unwrap()),
    };
    s.solution = d.reference.iter().map(encode).collect();
    s.goal = Goal::SmtpRouting {
        definition: Box::new(d),
    };
    s
}
