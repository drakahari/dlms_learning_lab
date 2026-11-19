//! Synthetic source fixture, not registered curriculum or native content acceptance.
#[path = "guest_driver_tasks.rs"]
mod task_data;
use dlms_learning_lab::{
    engine::{Goal, State},
    guest_driver_source as binding,
    guest_driver_workspace::{Action, Session},
    pack,
};
pub fn encode(a: &Action) -> String {
    format!("{}{}", binding::PREFIX, serde_json::to_string(a).unwrap())
}
pub fn scenario() -> pack::Scenario {
    let mut s = pack::load(include_str!(
        "../../scenarios/kernel-module-lessons-v1.json"
    ))
    .unwrap()
    .remove(0);
    let d = task_data::definition();
    s.id = d.source_id.clone();
    s.unit = "test-guest-image-unit".into();
    s.version = 1;
    s.title = "Repair the supplied guest image".into();
    s.objective="Activate the presented controller and restore its files and lookup for a fresh driver request. Retain the active audit provider and its prerequisite settings, preserve the kernel package version, and resolve installed package requirements.".into();
    s.initial = State::GuestDrivers {
        simulation: Box::new(Session::new(&d).unwrap()),
    };
    s.solution = d.reference.iter().map(encode).collect();
    for g in &mut s.guide {
        for c in &mut g.commands {
            c.command = c
                .command
                .replace("example_reader", "guide_reader")
                .replace("reader_bus", "guide_bus");
        }
    }
    s.goal = Goal::GuestDrivers {
        definition: Box::new(d),
    };
    s
}
