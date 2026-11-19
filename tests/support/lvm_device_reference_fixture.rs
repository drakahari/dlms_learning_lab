use dlms_learning_lab::{
    lvm,
    lvm_workspace::{self as w, device_reference as goals, device_reference_flow as f},
};
use std::collections::BTreeMap;
pub fn save(caller: &str, path: &str) -> f::Operation {
    f::Operation::SaveReference {
        caller: caller.into(),
        path: path.into(),
    }
}
pub fn command(text: &str) -> f::Operation {
    f::Operation::Command { text: text.into() }
}
fn context(g: &str, v: &str) -> f::Context {
    f::Context {
        initial: lvm::device_reference::Configuration {
            callers: BTreeMap::from([(
                "reporting".into(),
                lvm::device_reference::Caller {
                    label: "Reporting application".into(),
                    file: "/keep.txt".into(),
                    path: lvm::device_path::modeled_mapper(g, v).unwrap(),
                },
            )]),
        },
        plan: goals::Plan {
            targets: vec![goals::Target {
                caller: "reporting".into(),
                original_volume: format!("{g}/{v}"),
                canonical_required: true,
            }],
            renaming: None,
        },
    }
}
pub fn definition() -> w::Definition {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let mut d: w::Definition =
        serde_json::from_value(raw["scenarios"][0]["goal"]["definition"].clone()).unwrap();
    d.targets.clear();
    d.lesson.targets.clear();
    d.renaming = None;
    d.lesson.renaming = None;
    d.lesson.worked.clear();
    d.lesson.experiment.clear();
    d.lesson.repair.clear();
    let c = context("team", "draft");
    let p = context("studio", "notes");
    d.device_reference_task = Some(f::Task {
        context: c,
        reference: vec![save("reporting", "/dev/team/draft")],
    });
    d.lesson.device_reference_practice = Some(f::Practice {
        context: p,
        worked: vec![
            f::Step {
                operation: command("lvs"),
                explanation:
                    "Read the current group and volume names before editing the supplied reference."
                        .into(),
            },
            f::Step {
                operation: save("reporting", "/dev/studio/notes"),
                explanation:
                    "Save the recommended reference and inspect the actual virtual report.".into(),
            },
        ],
        experiment: save("reporting", "/dev/studio/missing"),
        repair: vec![save("reporting", "/dev/studio/notes")],
    });
    d
}
