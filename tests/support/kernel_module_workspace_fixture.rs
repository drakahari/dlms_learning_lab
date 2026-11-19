#[path = "kernel_module_fixture.rs"]
mod primitive;
use dlms_learning_lab::{kernel_module_workspace::*, kernel_modules as core};
pub fn definition(case: usize) -> Definition {
    fn activity(case: usize, practice: bool) -> Activity {
        let c0 = primitive::catalogue();
        let text = serde_json::to_string(&c0).unwrap();
        let text = if practice {
            text.replace("lab_sensor", "example_reader")
                .replace("sensor_bus", "reader_bus")
                .replace("bus_core", "reader_core")
                .replace("audit_link", "example_audit")
                .replace("old_sensor", "old_reader")
                .replace("6.8.0-lab", "6.9.0-example")
                .replace("6.7.0-lab", "6.8.0-example")
        } else {
            text
        };
        let catalogue: core::Catalogue = serde_json::from_str(&text).unwrap();
        let module = if practice {
            "example_reader"
        } else {
            "lab_sensor"
        };
        let audit = if practice {
            "example_audit"
        } else {
            "audit_link"
        };
        let mut initial = primitive::runtime(&catalogue);
        primitive::ok(&mut initial, &catalogue, &format!("modprobe {audit}"));
        let value = if practice { 3 } else { 4 };
        if case == 1 {
            primitive::ok(&mut initial, &catalogue, &format!("modprobe {module}"));
            initial.config =
                format!("# Repair only the target module\noptions {module} queues=99\n");
        }
        if case == 2 {
            initial.index.remove(module);
        }
        Activity {
            catalogue,
            initial,
            target: Target {
                module: module.into(),
                parameters: [("queues".into(), if case == 1 { value } else { 1 })].into(),
                saved_options: case == 1,
                current_index: case == 2,
                request_names: vec![],
            },
        }
    }
    fn steps(a: &Activity, case: usize) -> Vec<Step> {
        let n = &a.target.module;
        let mut out = vec![];
        if case == 1 {
            out.push(Step::SaveConfig {
                text: format!("options {n} queues={}\n", a.target.parameters["queues"]),
            });
            out.push(Step::Command {
                text: format!("modprobe -r {n}"),
            });
        }
        if case == 2 {
            out.push(Step::Command {
                text: "depmod -a".into(),
            });
        }
        out.push(Step::Command {
            text: format!("modprobe {n}"),
        });
        out
    }
    let assignment = activity(case, false);
    let example = activity(case, true);
    Definition {
        version: 1,
        source_id: format!("test-module-{case}"),
        reference: steps(&assignment, case),
        example_steps: steps(&example, case),
        assignment,
        example,
    }
}
