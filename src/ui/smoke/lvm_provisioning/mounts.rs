//! Fixed native editor/terminal mount tasks; no host device or mount command.
use super::*;
use work::mount_flow::Operation;
pub(in crate::ui) const IDS: [&str; 2] = ["lpic2-lvm-mount-repair", "lpic2-lvm-mounted-handover"];
fn apply(ops: &mut Vec<Op>, action: &Operation) {
    match action {
        Operation::Command { text } => command(ops, text),
        Operation::SavePolicy { text } => {
            ops.extend([Op::MountField(text.clone()), Op::Click("mount-save".into())])
        }
    }
}
pub(super) fn route(labs: &[pack::Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    for id in IDS {
        let source = labs.iter().find(|s| s.id == id).unwrap();
        let engine::Goal::Lvm { definition: d } = &source.goal else {
            panic!()
        };
        let p = d.lesson.mount_practice.as_ref().unwrap();
        let t = d.mount_task.as_ref().unwrap();
        ops.extend([
            Op::Open(id.into()),
            Op::Shot(format!("mount-{id}-introduction"), "reset-header".into()),
            Op::Click("practice-start".into()),
            Op::Unrun,
            Op::Solved(false),
            Op::Shot(format!("mount-{id}-guided-start"), "mount-editor".into()),
        ]);
        for step in &p.worked {
            apply(&mut ops, &step.operation);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Solved(false),
            Op::Shot(format!("mount-{id}-example-passed"), "check".into()),
        ]);
        apply(&mut ops, &p.experiment);
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(false),
            Op::Shot(format!("mount-{id}-example-mistake"), "check".into()),
        ]);
        for op in &p.repair {
            apply(&mut ops, op);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Click("assignment".into()),
            Op::Solved(false),
            Op::Shot(
                format!("mount-{id}-assignment-unstarted"),
                "mount-editor".into(),
            ),
            Op::MountField("# unfinished repair\n/dev/".into()),
            Op::Click("mount-save".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("mount-{id}-syntax-feedback"), "check".into()),
            Op::Save,
            Op::Fresh,
            Op::Solved(false),
        ]);
        for op in &t.reference {
            apply(&mut ops, op);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Shot(format!("mount-{id}-assignment-passed"), "check".into()),
            Op::Save,
            Op::Fresh,
            Op::Solved(true),
            Op::MountField("# unfinished unsaved draft\nUUID=".into()),
            Op::Solved(false),
            Op::Save,
            Op::Fresh,
            Op::Solved(false),
            Op::Shot(format!("mount-{id}-unsaved-resume"), "mount-editor".into()),
        ]);
        // Live/name work is already correct after resume. Repair only the draft;
        // replaying an old rename would correctly fail and is not a reset.
        for op in &t.reference {
            if matches!(op, Operation::SavePolicy { .. }) {
                apply(&mut ops, op);
            }
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Click("challenge-header".into()),
            Op::Challenge(true),
            Op::Shot(format!("mount-{id}-challenge"), "mount-editor".into()),
            Op::Click("learn-header".into()),
            Op::Challenge(false),
            Op::Theme(false),
            Op::Shot(format!("mount-{id}-dark200-editor"), "mount-editor".into()),
            Op::Shot(format!("mount-{id}-dark200-controls"), "run".into()),
            Op::Theme(true),
            Op::Shot(format!("mount-{id}-light150-editor"), "mount-editor".into()),
            Op::Shot(format!("mount-{id}-light150-controls"), "run".into()),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Solved(false),
        ]);
    }
    ops
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_mount_route_exercises_real_editor_and_terminal_partial_retry_resume_and_scales() {
        let labs = pack::ccna_pilot().unwrap();
        let ops = route(&labs);
        assert!((140..=240).contains(&ops.len()), "{}", ops.len());
        for id in IDS {
            assert!(ops.iter().any(|o| matches!(o,Op::Open(s)if s==id)));
        }
        for key in ["mount-save", "run", "check", "assignment", "confirm-reset"] {
            assert!(
                ops.iter().any(|o| matches!(o,Op::Click(s)if s==key)),
                "{key}"
            );
        }
        for prefix in [
            "blkid",
            "findmnt",
            "cat ",
            "umount ",
            "mount ",
            "vgrename ",
            "lvrename ",
        ] {
            assert!(
                ops.iter()
                    .any(|o| matches!(o,Op::Field(s)if s.starts_with(prefix))),
                "{prefix}"
            );
        }
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::MountField(s)if s.contains('\n')))
        );
        assert!(ops.iter().any(|o| matches!(o, Op::Practice(false))));
        assert!(ops.iter().any(|o| matches!(o, Op::Fresh)));
        assert!(ops.iter().any(|o| matches!(o, Op::Solved(true))));
        assert!(ops.iter().any(|o| matches!(o, Op::Challenge(true))));
    }
}
