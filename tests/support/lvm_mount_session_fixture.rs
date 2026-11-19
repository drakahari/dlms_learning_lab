//! Synthetic unregistered mount session/source fixture.
#![allow(dead_code)]
use dlms_learning_lab::{
    engine::{Goal, State},
    lvm_workspace::{self as w, mount_flow as f},
    pack::{self, Pack},
};
#[path = "lvm_mount_fixture.rs"]
pub mod model;
fn example_world() -> dlms_learning_lab::lvm::State {
    let raw = serde_json::to_string(&model::world())
        .unwrap()
        .replace("team", "demo")
        .replace("reserve", "spares");
    serde_json::from_str(&raw).unwrap()
}
fn example_practice() -> f::Practice {
    let p = model::practice();
    serde_json::from_str(
        &serde_json::to_string(&p)
            .unwrap()
            .replace("team", "demo")
            .replace("/srv/report", "/srv/practice"),
    )
    .unwrap()
}
pub fn definition() -> w::Definition {
    let p: serde_json::Value =
        serde_json::from_str(include_str!("../../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let mut d: w::Definition =
        serde_json::from_value(p["scenarios"][0]["goal"]["definition"].clone()).unwrap();
    d.targets.clear();
    d.plan = None;
    d.retirement = None;
    d.availability = None;
    d.renaming = None;
    d.device_reference_task = None;
    d.snapshot_task = None;
    d.config_scan_task = None;
    d.mount_task = Some(model::task());
    d.lesson.fixture = example_world();
    d.lesson.targets.clear();
    d.lesson.plan = None;
    d.lesson.retirement = None;
    d.lesson.availability = None;
    d.lesson.renaming = None;
    d.lesson.worked.clear();
    d.lesson.snapshot_practice = None;
    d.lesson.device_reference_practice = None;
    d.lesson.config_scan_practice = None;
    d.lesson.mount_practice = Some(example_practice());
    d.lesson.experiment.clear();
    d.lesson.repair.clear();
    d
}
pub fn scenario() -> pack::Scenario {
    let raw: Pack =
        serde_json::from_str(include_str!("../../scenarios/lvm-growth-v1.json")).unwrap();
    let mut s = raw.scenarios[0].clone();
    let d = definition();
    s.id = "lpic2-test-only-mount".into();
    s.unit = "test-only-mount".into();
    s.prerequisites.clear();
    s.title = "Repair a supplied mount".into();
    s.objective="Save the intended filesystem entry, replace the wrong live mount and verify its original file.".into();
    s.solution = d
        .mount_task
        .as_ref()
        .unwrap()
        .reference
        .iter()
        .map(|op| op.description())
        .collect();
    s.initial = State::Lvm {
        simulation: Box::new(w::Session::new(&d).unwrap()),
    };
    s.goal = Goal::Lvm {
        definition: Box::new(d),
    };
    s
}
