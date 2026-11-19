use dlms_learning_lab::{engine::State, pack, partition_fs_workspace as work, progress};
use std::sync::OnceLock;
use work::Operation as Op;
const MIB: u64 = 1024 * 1024;
const S: u64 = 2048;
fn lessons() -> &'static Vec<pack::Scenario> {
    static L: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    L.get_or_init(|| {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v14.json")).unwrap();
        data["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["id"].as_str().unwrap().starts_with("lpic1-layout-"))
            .map(|s| serde_json::from_value(s.clone()).unwrap())
            .collect()
    })
}
fn def(s: &pack::Scenario) -> &work::Definition {
    work::binding::source(s).unwrap()
}
fn add(disk: &str, name: &str, start: u64, size: u64, swap: bool) -> Op {
    Op::AddPartition {
        disk: disk.into(),
        name: name.into(),
        start: start * S,
        end: (start + size) * S - 1,
        type_hint: if swap {
            dlms_learning_lab::partition_fs::TypeHint::LinuxSwap
        } else {
            dlms_learning_lab::partition_fs::TypeHint::LinuxData
        },
    }
}
fn format(disk: &str, n: u8, name: &str, swap: bool) -> Op {
    Op::Format {
        partition: format!("{disk}{n}"),
        format: if swap {
            work::FormatChoice::Swap
        } else {
            work::FormatChoice::Ext4
        },
        label: name.into(),
        uuid: None,
    }
}
fn apply(a: &mut progress::Attempt, s: &pack::Scenario, ops: &[Op], practice: bool) {
    for op in ops {
        work::binding::apply(a, s, op, practice).unwrap();
    }
}
fn session(a: &progress::Attempt) -> &work::Session {
    let State::PartitionFs { simulation } = &a.state else {
        panic!()
    };
    simulation
}
fn role_ops(s: &pack::Scenario, root: &str, logs: &str, swap: &str, swap_start: u64) -> Vec<Op> {
    let mut ops = vec![Op::CreateGpt {
        disk: "/dev/vdb".into(),
    }];
    let rootstart = if root == "/dev/vda" { 5 } else { 1 };
    let logstart = if logs == "/dev/vda" {
        if root == logs { 21 } else { 5 }
    } else {
        1
    };
    let rootn = if root == "/dev/vda" { 2 } else { 1 };
    let logn = if logs == root {
        rootn + 1
    } else if logs == "/dev/vda" {
        2
    } else {
        1
    };
    ops.extend([
        add(root, "root", rootstart, 16, false),
        format(root, rootn, "root", false),
        add(logs, "logs", logstart, 12, false),
        format(logs, logn, "logs", false),
    ]);
    let supplied = def(s).initial.disks[swap].partitions().count();
    let added = ops
        .iter()
        .filter(|op| matches!(op,Op::AddPartition{disk,..}if disk==swap))
        .count();
    let swapn = u8::try_from(supplied + added + 1).unwrap();
    ops.extend([
        add(swap, "swap", swap_start, 4, true),
        format(swap, swapn, "swap", true),
    ]);
    assert!(def(s).plan.layout.is_some());
    ops
}
#[test]
fn both_sources_taught_repairs_and_separate_native_examples_are_reachable() {
    assert_eq!(lessons().len(), 2);
    for s in lessons() {
        let d = def(s);
        d.validate().unwrap();
        let mut a = progress::Attempt::new(s, 1);
        assert!(d.fresh(session(&a)));
        assert!(!progress::current_solved(&a, s));
        let original = session(&a).current.clone();
        let ops = d
            .lesson
            .worked
            .iter()
            .map(|step| step.operation.clone())
            .collect::<Vec<_>>();
        apply(&mut a, s, &ops, true);
        assert!(d.practice_checks(session(&a)).iter().all(|c| c.0));
        assert_eq!(session(&a).current, original);
        assert!(!progress::current_solved(&a, s));
        apply(&mut a, s, &d.reference, false);
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn valid_log_extents_pass_at_multiple_locations_and_sizes_while_earlier_free_space_fails() {
    let s = &lessons()[0];
    let d = def(s);
    for (start, size) in [(28, 8), (30, 12), (32, 10)] {
        let mut a = progress::Attempt::new(s, 1);
        apply(
            &mut a,
            s,
            &[
                add("/dev/vda", "logs", start, size, false),
                format("/dev/vda", 2, "logs", false),
            ],
            false,
        );
        assert!(progress::current_solved(&a, s));
    }
    let mut a = progress::Attempt::new(s, 1);
    apply(
        &mut a,
        s,
        &[
            add("/dev/vda", "logs", 1, 8, false),
            format("/dev/vda", 2, "logs", false),
        ],
        false,
    );
    let checks = d.checks(session(&a));
    assert_eq!(
        checks.iter().map(|c| c.0).collect::<Vec<_>>(),
        [true, true, false]
    );
    assert!(checks[2].1.contains("11534336"));
    assert!(!progress::current_solved(&a, s));
}
#[test]
fn independent_media_and_adjacent_room_are_distinct_behavior_conditions() {
    let s = &lessons()[1];
    let d = def(s);
    for (root, logs, swap, start) in [
        ("/dev/vda", "/dev/vdb", "/dev/vda", 21),
        ("/dev/vdb", "/dev/vda", "/dev/vdb", 17),
    ] {
        let mut a = progress::Attempt::new(s, 1);
        apply(&mut a, s, &role_ops(s, root, logs, swap, start), false);
        assert!(progress::current_solved(&a, s));
    }
    let mut same = progress::Attempt::new(s, 1);
    apply(
        &mut same,
        s,
        &role_ops(s, "/dev/vda", "/dev/vda", "/dev/vdb", 1),
        false,
    );
    let checks = d.checks(session(&same));
    assert_eq!(
        checks.iter().map(|c| c.0).collect::<Vec<_>>(),
        [true, true, true, true, true, false]
    );
    let mut blocked = progress::Attempt::new(s, 1);
    apply(
        &mut blocked,
        s,
        &role_ops(s, "/dev/vda", "/dev/vdb", "/dev/vdb", 13),
        false,
    );
    let checks = d.checks(session(&blocked));
    assert_eq!(
        checks.iter().map(|c| c.0).collect::<Vec<_>>(),
        [true, true, true, true, false, true]
    );
}
#[test]
fn partial_unformatted_work_and_later_window_consumption_cannot_keep_success() {
    for s in lessons() {
        let d = def(s);
        let mut a = progress::Attempt::new(s, 1);
        for op in d
            .reference
            .iter()
            .filter(|op| !matches!(op, Op::Format { .. }))
        {
            work::binding::apply(&mut a, s, op, false).unwrap();
        }
        assert!(!progress::current_solved(&a, s));
        assert!(d.checks(session(&a)).iter().any(|c| c.0));
        assert!(
            d.checks(session(&a))
                .iter()
                .any(|c| !c.0 && c.1.contains("content"))
        );
        let mut a = progress::Attempt::new(s, 1);
        apply(&mut a, s, &d.reference, false);
        assert!(progress::current_solved(&a, s));
        let (disk, start) = if s.id.ends_with("growth") {
            ("/dev/vda", 37)
        } else {
            ("/dev/vdb", 18)
        };
        apply(&mut a, s, &[add(disk, "scratch", start, 1, false)], false);
        assert!(!progress::current_solved(&a, s));
        assert!(
            d.checks(session(&a))
                .iter()
                .any(|c| !c.0 && c.1.contains("unused directly"))
        );
    }
}
#[test]
fn authored_reachability_forged_origins_and_wrong_rule_versions_are_rejected() {
    let s = &lessons()[0];
    let d = def(s);
    let mut bad = d.clone();
    bad.reference = vec![
        add("/dev/vda", "logs", 1, 8, false),
        format("/dev/vda", 2, "logs", false),
    ];
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.plan.layout.as_mut().unwrap().version = 2;
    assert!(bad.validate().is_err());
    let mut a = progress::Attempt::new(s, 1);
    let State::PartitionFs { simulation } = &mut a.state else {
        panic!()
    };
    simulation.origin.plan.layout = None;
    assert!(work::binding::context(&a, s).is_err());
    assert!(!progress::current_solved(&a, s));
}
#[test]
fn save_resume_practice_reset_and_selected_attempt_reset_preserve_other_work() {
    let mut catalog = pack::accepted394().unwrap();
    assert_eq!(catalog.len(), 394);
    catalog.extend(lessons().clone());
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in lessons() {
        let d = def(s);
        let mut a = progress::Attempt::new(s, 1);
        apply(&mut a, s, &d.reference, false);
        let mut draft = session(&a)
            .example_editor
            .drafts
            .values()
            .next()
            .unwrap()
            .clone();
        draft.name = "unfinished-practice".into();
        let disk = session(&a).example_editor.selected.clone();
        work::binding::set_draft(&mut a, s, &disk, draft, true).unwrap();
        assert!(progress::current_solved(&a, s));
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let mut loaded = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    let s = &lessons()[0];
    work::binding::reset_example(loaded.attempts.get_mut(&s.id).unwrap(), s).unwrap();
    assert!(progress::current_solved(&loaded.attempts[&s.id], s));
    let keep = serde_json::to_value(&loaded.attempts[&lessons()[1].id]).unwrap();
    loaded
        .attempts
        .insert(s.id.clone(), progress::Attempt::new(s, 2));
    assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&lessons()[1].id]).unwrap(),
        keep
    );
    assert_eq!(
        def(&lessons()[0]).plan.layout.as_ref().unwrap().rules.len(),
        1
    );
    assert_eq!(
        def(&lessons()[1]).plan.layout.as_ref().unwrap().rules.len(),
        2
    );
    assert!(MIB.is_multiple_of(512));
}

#[test]
fn registered_catalog_and_unit_extend_every_prior_value_without_inventing_progress() {
    let old = pack::accepted394().unwrap();
    let current = pack::accepted396().unwrap();
    assert_eq!((old.len(), current.len()), (394, 396));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&current[..394]).unwrap()
    );
    let before = dlms_learning_lab::course::accepted394(&old).unwrap();
    let units = dlms_learning_lab::course::accepted396(&current).unwrap();
    assert_eq!((before.len(), units.len()), (151, 152));
    let retained: Vec<_> = units
        .iter()
        .filter(|u| u.id != "linux-workload-disk-layout")
        .collect();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-workload-disk-layout")
        .unwrap();
    assert_eq!(units[at - 1].id, "linux-disk-preparation");
}
