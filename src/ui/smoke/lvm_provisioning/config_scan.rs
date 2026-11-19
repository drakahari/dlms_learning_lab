//! Fixed native editor/selection/scan route. No host LVM or learner script.
use super::*;
use std::collections::BTreeSet;
use work::config_scan::{Operation, Selection};
pub(in crate::ui) const IDS: [&str; 2] = ["lpic2-lvm-filter-policy", "lpic2-lvm-devices-file"];
fn selected(file: &Option<Selection>) -> BTreeSet<String> {
    file.iter()
        .flat_map(|f| f.entries.iter().map(|e| e.0.clone()))
        .collect()
}
fn apply(ops: &mut Vec<Op>, action: &Operation, current: &mut BTreeSet<String>) {
    match action {
        Operation::SaveConfig { text } => ops.extend([
            Op::ConfigField(text.0.clone()),
            Op::Click("save-config".into()),
        ]),
        Operation::SaveSelection { selection } => {
            let want = selected(&Some(selection.clone()));
            for path in current.symmetric_difference(&want) {
                ops.push(Op::Click(format!("select-{path}")));
            }
            ops.push(Op::Click("save-selection".into()));
            *current = want;
        }
        Operation::Scan => ops.push(Op::Click("run".into())),
    }
}
pub(super) fn route(labs: &[pack::Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    for id in IDS {
        let s = labs.iter().find(|s| s.id == id).unwrap();
        let engine::Goal::Lvm { definition: d } = &s.goal else {
            panic!()
        };
        let p = d.lesson.config_scan_practice.as_ref().unwrap();
        let t = d.config_scan_task.as_ref().unwrap();
        ops.extend([
            Op::Open(id.into()),
            Op::Shot(format!("config-{id}-introduction"), "reset-header".into()),
            Op::Click("practice-start".into()),
            Op::Unrun,
            Op::Solved(false),
            Op::Shot(format!("config-{id}-guided-start"), "config-editor".into()),
            Op::Click("example-worked".into()),
            Op::Click("fill-config".into()),
        ]);
        let mut example = selected(&p.context.initial_file);
        if p.context.initial_file.is_some() {
            ops.push(Op::Click("fill-selection".into()));
            example = selected(&match &p
                .worked
                .iter()
                .find(|s| matches!(s.operation, Operation::SaveSelection { .. }))
                .unwrap()
                .operation
            {
                Operation::SaveSelection { selection } => Some(selection.clone()),
                _ => unreachable!(),
            });
        }
        ops.extend([
            Op::Unrun,
            Op::Click("example-worked".into()),
            Op::Click("save-config".into()),
        ]);
        if p.context.initial_file.is_some() {
            ops.push(Op::Click("save-selection".into()));
        }
        ops.extend([
            Op::Click("run".into()),
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Solved(false),
            Op::Shot(format!("config-{id}-example-passed"), "scan-0-0".into()),
        ]);
        for action in &p.experiment {
            apply(&mut ops, action, &mut example);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(false),
            Op::Shot(format!("config-{id}-example-mistake"), "check".into()),
        ]);
        for action in &p.repair {
            apply(&mut ops, action, &mut example);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Click("assignment".into()),
            Op::Solved(false),
            Op::Shot(
                format!("config-{id}-assignment-unstarted"),
                "config-editor".into(),
            ),
            Op::ConfigField("devices {".into()),
            Op::Click("save-config".into()),
            Op::Click("run".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("config-{id}-syntax-feedback"), "scan-error".into()),
            Op::Save,
            Op::Fresh,
            Op::Solved(false),
        ]);
        let mut assignment = selected(&t.context.initial_file);
        for action in &t.reference {
            apply(&mut ops, action, &mut assignment);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Shot(format!("config-{id}-assignment-passed"), "check".into()),
            Op::Save,
            Op::Fresh,
            Op::Solved(true),
            Op::ConfigField("# unfinished saved draft\ndevices {".into()),
            Op::Solved(false),
            Op::Save,
            Op::Fresh,
            Op::Solved(false),
            Op::Shot(
                format!("config-{id}-unsaved-resume"),
                "config-editor".into(),
            ),
        ]);
        for action in &t.reference {
            apply(&mut ops, action, &mut assignment);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Click("challenge-header".into()),
            Op::Challenge(true),
            Op::Shot(format!("config-{id}-challenge"), "config-editor".into()),
            Op::Click("learn-header".into()),
            Op::Challenge(false),
            Op::Theme(false),
            Op::Shot(
                format!("config-{id}-dark200-editor"),
                "config-editor".into(),
            ),
            Op::Shot(format!("config-{id}-dark200-controls"), "run".into()),
            Op::Theme(true),
            Op::Shot(
                format!("config-{id}-light150-editor"),
                "config-editor".into(),
            ),
            Op::Shot(format!("config-{id}-light150-controls"), "run".into()),
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
    fn fixed_config_route_contains_both_real_editors_saves_scan_errors_and_independent_cases() {
        let labs = pack::ccna_pilot().unwrap();
        let ops = route(&labs);
        assert!((100..=240).contains(&ops.len()), "{}", ops.len());
        for id in IDS {
            assert!(ops.iter().any(|o| matches!(o,Op::Open(v)if v==id)));
        }
        for key in [
            "fill-config",
            "fill-selection",
            "save-config",
            "save-selection",
            "run",
            "check",
            "select-/dev/import-copy",
        ] {
            assert!(
                ops.iter().any(|o| matches!(o,Op::Click(v)if v==key)),
                "{key}"
            );
        }
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::ConfigField(v)if v.contains('\n')))
        );
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Shot(_,target)if target=="scan-error"))
        );
        assert!(ops.iter().any(|o| matches!(o, Op::Practice(false))));
        assert!(ops.iter().any(|o| matches!(o, Op::Solved(true))));
        assert!(ops.iter().any(|o| matches!(o, Op::Fresh)));
    }
}
