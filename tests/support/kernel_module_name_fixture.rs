#[path = "kernel_module_workspace_fixture.rs"]
mod physical;
use dlms_learning_lab::kernel_module_workspace::*;
pub fn definition() -> Definition {
    let mut d = physical::definition(0);
    d.source_id = "test-named-module-handover".into();
    for (a, practice) in [(&mut d.assignment, false), (&mut d.example, true)] {
        let (current, legacy, audit_name, audit) = if practice {
            ("demo_current", "demo_legacy", "demo_audit", "example_audit")
        } else {
            (
                "capture_current",
                "capture_legacy",
                "audit_query",
                "audit_link",
            )
        };
        a.target.request_names = vec![current.into(), legacy.into()];
        a.initial.config = format!(
            "alias {current} {audit}\nalias {legacy} {}\nalias {audit_name} {audit}\n",
            a.target.module
        );
    }
    fn steps(a: &Activity, practice: bool) -> Vec<Step> {
        let (current, legacy, audit_name, audit) = if practice {
            ("demo_current", "demo_legacy", "demo_audit", "example_audit")
        } else {
            (
                "capture_current",
                "capture_legacy",
                "audit_query",
                "audit_link",
            )
        };
        vec![
            Step::SaveConfig {
                text: format!(
                    "alias {current} {}\nalias {legacy} {}\nalias {audit_name} {audit}\n",
                    a.target.module, a.target.module
                ),
            },
            Step::Command {
                text: format!("modprobe {current}"),
            },
        ]
    }
    d.reference = steps(&d.assignment, false);
    d.example_steps = steps(&d.example, true);
    d
}
