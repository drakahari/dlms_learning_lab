//! Fixed real native controls over the registered COW unit; never host LVM.
use super::*;
use dlms_learning_lab::lvm_workspace::snapshot_flow::Operation;
pub(in crate::ui) const IDS: [&str; 3] = [
    "lpic2-lvm-saved-view",
    "lpic2-lvm-cow-budget",
    "lpic2-lvm-record-recovery",
];
fn record(ops: &mut Vec<Op>, origin: &str, record: &str) {
    ops.extend([
        Op::Click(format!("snapshot-record-{origin}")),
        Op::Click(format!("snapshot-record-choice-{origin}-{record}")),
    ]);
}
fn apply(ops: &mut Vec<Op>, op: &Operation) {
    match op {
        Operation::Command { text } => command(ops, text),
        Operation::ApplyUpdate { index } => ops.push(Op::Click(format!("snapshot-update-{index}"))),
        Operation::CopyRecord {
            origin,
            snapshot,
            record: r,
        } => {
            record(ops, origin, r);
            ops.push(Op::Click(format!("snapshot-copy-{origin}-{snapshot}")));
        }
    }
}
fn example(ops: &mut Vec<Op>, d: &work::Definition) {
    for (index, step) in d
        .lesson
        .snapshot_practice
        .as_ref()
        .unwrap()
        .worked
        .iter()
        .enumerate()
    {
        ops.extend([
            Op::Click("example-picker".into()),
            Op::Click(format!("example-choice-{index}")),
            Op::Click("insert".into()),
        ]);
        if matches!(step.operation, Operation::Command { .. }) {
            ops.push(Op::Click("run".into()));
        }
    }
}
pub(super) fn route(labs: &[pack::Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    for (n, id) in IDS.iter().enumerate() {
        let s = labs.iter().find(|s| s.id == *id).unwrap();
        let engine::Goal::Lvm { definition: d } = &s.goal else {
            panic!()
        };
        let p = d.lesson.snapshot_practice.as_ref().unwrap();
        let task = d.snapshot_task.as_ref().unwrap();
        ops.extend([
            Op::Open((*id).into()),
            Op::Shot(format!("cow-{id}-introduction"), "practice-start".into()),
            Op::Click("practice-start".into()),
            Op::Click("insert".into()),
        ]);
        // First recovery step is a clearly labeled explicit copy; first other
        // steps are inert command insertion. Do not pretend copy is inert.
        if n < 2 {
            ops.push(Op::Unrun);
            ops.push(Op::Click("run".into()));
        }
        for (index, step) in p.worked.iter().enumerate().skip(1) {
            ops.extend([
                Op::Click("example-picker".into()),
                Op::Click(format!("example-choice-{index}")),
                Op::Click("insert".into()),
            ]);
            if matches!(step.operation, Operation::Command { .. }) {
                ops.push(Op::Click("run".into()));
            }
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Solved(false),
            Op::Shot(format!("cow-{id}-practice-pass"), "check".into()),
        ]);
        apply(&mut ops, &p.experiment);
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(false),
            Op::Shot(format!("cow-{id}-practice-mistake"), "check".into()),
        ]);
        for op in &p.repair {
            apply(&mut ops, op);
        }
        if !p.repair.is_empty() {
            ops.extend([Op::Click("check".into()), Op::Practice(true)]);
        }
        ops.push(Op::Click("reset-example".into()));
        example(&mut ops, d);
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Click("assignment".into()),
            Op::Shot(format!("cow-{id}-assignment-unstarted"), "check".into()),
            Op::Click("check".into()),
            Op::Solved(false),
        ]);
        command(&mut ops, "sh -c lvs");
        ops.push(Op::Notice);
        // One real step is partial; save/load must retain it without credit.
        apply(&mut ops, &task.reference[0]);
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Save,
            Op::Fresh,
            Op::Solved(false),
            Op::Shot(format!("cow-{id}-partial"), "check".into()),
        ]);
        for op in task.reference.iter().skip(1) {
            apply(&mut ops, op);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Fresh,
            Op::Solved(true),
            Op::Shot(format!("cow-{id}-assignment-pass"), "check".into()),
        ]);
        if n == 0 {
            record(&mut ops, "report/report", "status");
            ops.extend([
                Op::RecordRead("report/report:current:status".into(), "week2=ok".into()),
                Op::RecordRead("report/report:before_run:status".into(), "week1=ok".into()),
                Op::Shot(
                    "cow-earlier-and-current-records".into(),
                    "report/report:before_run:status".into(),
                ),
            ]);
        }
        let g = ["report", "budget", "recovery"][n];
        command(
            &mut ops,
            &format!("lvcreate -s -L 1M -n extra -c 128 {g}/report"),
        );
        ops.extend([
            Op::Solved(false),
            Op::Click("check".into()),
            Op::Shot(format!("cow-{id}-invalidated"), "check".into()),
        ]);
        command(&mut ops, &format!("lvremove -y {g}/extra"));
        ops.push(Op::Solved(true));
        for light in [true, false] {
            ops.push(Op::Theme(light));
            command(&mut ops, "lvs");
            ops.push(Op::Shot(
                format!(
                    "cow-{id}-{}-{}-field",
                    if light { "light" } else { "dark" },
                    if light { 150 } else { 200 }
                ),
                "command".into(),
            ));
        }
        ops.extend([
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Solved(false),
            Op::Click("challenge-header".into()),
            Op::Challenge(true),
            Op::Shot(format!("cow-{id}-challenge"), "check".into()),
        ]);
        for op in &task.reference {
            apply(&mut ops, op);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Click("learn-header".into()),
            Op::Challenge(false),
            Op::Save,
        ]);
    }
    ops
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cow_native_route_uses_real_helpers_and_commands_before_final_rendered_preview() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&a.scenarios);
        assert_eq!(ops.iter().filter(|op| matches!(op, Op::Open(_))).count(), 3);
        assert_eq!(ops.iter().filter(|op| matches!(op, Op::Fresh)).count(), 6);
        assert!(ops.iter().any(|op| matches!(op, Op::RecordRead(..))));
        // Preview readiness must never clear Challenge on behalf of the learner.
        // Every real route returns through the actual Learn control before handing
        // that saved attempt to the Introduction/Example screenshot phase.
        for section in ops
            .split(|op| matches!(op, Op::Open(_)))
            .filter(|s| !s.is_empty())
        {
            assert!(matches!(section.last(), Some(Op::Save)));
            assert!(matches!(
                section.get(section.len() - 2),
                Some(Op::Challenge(false))
            ));
            assert!(
                matches!(section.get(section.len()-3),Some(Op::Click(key))if key=="learn-header")
            );
        }

        assert!(
            ops.iter()
                .any(|op| matches!(op,Op::Click(key)if key.starts_with("snapshot-copy-")))
        );
        assert!(!ops.iter().any(
            |op| matches!(op,Op::Field(text)if text.starts_with(work::snapshot_flow::PREFIX))
        ));
    }
}
