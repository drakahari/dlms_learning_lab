#[path = "guest_driver_fixture.rs"]
mod fixture;
use dlms_learning_lab::{guest_driver_workspace::*, rpm_packages as rpm};
pub fn definition() -> Definition {
    let f = fixture::facts();
    let r = fixture::runtime(&f);
    let task = Task {
        facts: f,
        initial: r,
        devices: vec!["00:05.0".into()],
        protected_drivers: vec!["audit_link".into()],
        protected_packages: vec![rpm::Identity {
            name: "kernel-core".into(),
            arch: rpm::Architecture::X86_64,
        }],
    };
    // Separate source values, never a shared runtime or assignment answer fill.
    let raw = serde_json::to_string(&task)
        .unwrap()
        .replace("lab_sensor", "guide_reader")
        .replace("audit_link", "guide_audit")
        .replace("sensor_bus", "guide_bus")
        .replace("bus_core", "guide_core")
        .replace("6.8.0", "6.9.0")
        .replace("-lab", "-example")
        .replace("guest-driver-fixture", "guide-driver-fixture")
        .replace("Teaching sensor controller", "Guide reader controller");
    let example: Task = serde_json::from_str(&raw).unwrap();
    fn steps(name: &str) -> Vec<Action> {
        vec![
            Action::Package {
                action: rpm::Action::InstallSet {
                    archives: vec!["support-current".into()],
                },
            },
            run("depmod -a"),
            run(&format!("modprobe {name}")),
            Action::Check,
        ]
    }
    Definition {
        version: 1,
        source_id: "test-guest-driver".into(),
        assignment: task,
        example,
        reference: steps("lab_sensor"),
        example_steps: steps("guide_reader"),
    }
}
pub fn run(s: &str) -> Action {
    Action::Run { input: s.into() }
}
