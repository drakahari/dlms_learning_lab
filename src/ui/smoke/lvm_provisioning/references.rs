//! Fixed native draft/Save/reader route; no host LVM or original profile.
use super::*;
use dlms_learning_lab::lvm_workspace::device_reference_flow::Operation;
pub(in crate::ui) const IDS: [&str; 2] =
    ["lpic2-lvm-reference-repair", "lpic2-lvm-reference-handover"];
fn apply(ops: &mut Vec<Op>, o: &Operation) {
    match o {
        Operation::Command { text } => command(ops, text),
        Operation::SaveReference { caller, path } => ops.extend([
            Op::ReferenceField(caller.clone(), path.clone()),
            Op::Click(format!("reference-save-{caller}")),
        ]),
    }
}
fn example(ops: &mut Vec<Op>, d: &work::Definition) {
    for (index, step) in d
        .lesson
        .device_reference_practice
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
        let p = d.lesson.device_reference_practice.as_ref().unwrap();
        let t = d.device_reference_task.as_ref().unwrap();
        ops.extend([
            Op::Open((*id).into()),
            Op::Shot(
                format!("references-{id}-introduction"),
                "practice-start".into(),
            ),
            Op::Click("practice-start".into()),
            Op::Click("insert".into()),
            Op::Unrun,
            Op::Click("run".into()),
        ]);
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
            Op::Shot(
                format!("references-{id}-example-pass"),
                "reference-reader-reporting".into(),
            ),
        ]);
        apply(&mut ops, &p.experiment);
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(false),
            Op::Shot(format!("references-{id}-example-mistake"), "check".into()),
        ]);
        for o in &p.repair {
            apply(&mut ops, o);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Click("reset-example".into()),
        ]);
        example(&mut ops, d);
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Click("assignment".into()),
            Op::Shot(
                format!("references-{id}-assignment-unstarted"),
                "reference-reporting".into(),
            ),
            Op::Click("check".into()),
            Op::Solved(false),
        ]);
        command(&mut ops, "dmsetup splitname guessed-name");
        ops.push(Op::Notice);
        apply(&mut ops, &t.reference[0]);
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Save,
            Op::Fresh,
            Op::Solved(false),
            Op::Shot(format!("references-{id}-partial"), "check".into()),
        ]);
        for o in t.reference.iter().skip(1) {
            apply(&mut ops, o);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Fresh,
            Op::Solved(true),
        ]);
        if n == 0 {
            ops.extend([
                Op::Click("optional-help".into()),
                Op::Click("worked-approach".into()),
                Op::Shot(
                    "references-worked-help-human-actions".into(),
                    "reference-worked-line-0".into(),
                ),
                Op::Click("worked-approach".into()),
                Op::Click("optional-help".into()),
            ]);
        }
        let (good, content) = if n == 0 {
            (
                "/dev/north-data/daily-report",
                "north-data/daily-report: retain this original record.\n",
            )
        } else {
            (
                "/dev/records-main/ledger-live",
                "store-main/ledger-old: retain this original record.\n",
            )
        };
        ops.extend([
            Op::ReferenceRead("reporting".into(), content.into()),
            Op::Shot(
                format!("references-{id}-reader-pass"),
                "reference-reader-reporting".into(),
            ),
            Op::ReferenceField("reporting".into(), "/dev/missing/wrong".into()),
            Op::Solved(false),
            Op::Save,
            Op::Fresh,
            Op::Solved(false),
            Op::Shot(
                format!("references-{id}-unsaved-resume"),
                "reference-reporting".into(),
            ),
            Op::Click("reference-save-reporting".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(
                format!("references-{id}-failed-reader"),
                "reference-reader-reporting".into(),
            ),
            Op::ReferenceField("reporting".into(), good.into()),
            Op::Click("reference-save-reporting".into()),
            Op::Click("check".into()),
            Op::Solved(true),
        ]);
        for light in [true, false] {
            ops.push(Op::Theme(light));
            ops.push(Op::ReferenceField("reporting".into(), good.into()));
            ops.push(Op::Shot(
                format!(
                    "references-{id}-{}-field",
                    if light { "light150" } else { "dark200" }
                ),
                "reference-reporting".into(),
            ));
        }
        ops.extend([
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Solved(false),
            Op::Click("challenge-header".into()),
            Op::Challenge(true),
            Op::Shot(
                format!("references-{id}-challenge"),
                "reference-reporting".into(),
            ),
        ]);
        for o in &t.reference {
            apply(&mut ops, o);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Click("learn-header".into()),
            Op::Challenge(false),
        ]);
    }
    ops
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_requires_explicit_saves_partial_and_invalidation_and_ends_in_actual_learn() {
        let labs = pack::ccna_pilot().unwrap();
        let r = route(&labs);
        assert_eq!(r.iter().filter(|o| matches!(o, Op::Open(_))).count(), 2);
        assert!(
            r.iter()
                .filter(|o| matches!(o, Op::ReferenceField(_, _)))
                .count()
                >= 10
        );
        assert!(r.iter().filter(|o| matches!(o, Op::Fresh)).count() >= 6);
        assert!(
            r.iter()
                .any(|o| matches!(o,Op::ReferenceRead(_,s)if s.contains("store-main/ledger-old")))
        );
        assert!(matches!(r.last(), Some(Op::Challenge(false))));
    }
}
