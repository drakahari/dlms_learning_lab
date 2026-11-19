//! Three authored MBR tasks; native and novice acceptance are separate.
use dlms_learning_lab::{
    backup,
    engine::{self, Goal, State},
    pack,
    partition_fs::{
        self as core,
        mbr::{Action, Kind},
    },
    partition_fs_workspace as work,
    progress::{self, Attempt, Progress},
    script_teaching::View,
};
use serde_json::{Value, json};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic1-mbr-primary-plan",
    "lpic1-mbr-logical-space",
    "lpic1-mbr-storage-handover",
];
#[test]
fn registered326_preserves_every323_scenario110_units_and_their_order() {
    let before = pack::accepted323().unwrap();
    let current = pack::accepted326().unwrap();
    assert_eq!((before.len(), current.len()), (323, 326));
    assert_eq!(
        serde_json::to_value(&current[..323]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        current[323..]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| current.iter().filter(|s| s.track == t).count()),
        [74, 114, 61, 77]
    );
    for (offset, s) in current[323..].iter().enumerate() {
        for id in &s.prerequisites {
            assert!(current.iter().position(|p| &p.id == id).unwrap() < 323 + offset);
        }
    }
    let old = dlms_learning_lab::course::accepted323(&before).unwrap();
    let units = dlms_learning_lab::course::accepted326(&current).unwrap();
    assert_eq!((old.len(), units.len()), (110, 111));
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "mbr-layouts")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| units.iter().filter(|u| u.track == t).count()),
        [20, 44, 21, 26]
    );
    let at = units.iter().position(|u| u.id == "mbr-layouts").unwrap();
    assert_eq!(units[at].labs, IDS);
    assert_eq!(units[at - 1].id, "linux-disk-preparation");
}
fn all() -> &'static Vec<pack::Scenario> {
    static ALL: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    ALL.get_or_init(|| {
        let before = pack::accepted323().unwrap();
        assert_eq!(before.len(), 323);
        let mut text = before
            .chunks(8)
            .map(|r| json!({"schema_version":1,"scenarios":r}).to_string())
            .collect::<Vec<_>>();
        text.push(include_str!("../scenarios/mbr-layout-lessons-v1.json").into());
        pack::load_catalog(&text.iter().map(String::as_str).collect::<Vec<_>>()).unwrap()
    })
}
fn labs() -> Vec<&'static pack::Scenario> {
    all()
        .iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
        .collect()
}
fn def(s: &pack::Scenario) -> &work::Definition {
    let Goal::PartitionFs { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn sim(a: &Attempt) -> &work::Session {
    let State::PartitionFs { simulation } = &a.state else {
        panic!()
    };
    simulation
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 8);
    work::binding::prepare(&mut a, s).unwrap();
    a
}
fn solve(s: &pack::Scenario) -> Attempt {
    let mut a = fresh(s);
    work::binding::set_view(&mut a, s, View::Assignment).unwrap();
    for op in &def(s).reference {
        work::binding::apply(&mut a, s, op, false).unwrap();
    }
    work::binding::check(&mut a, s, false).unwrap();
    assert!(progress::current_solved(&a, s));
    a
}
fn add(role: Kind, start: u64, end: u64, swap: bool) -> work::Operation {
    work::Operation::Mbr {
        operation: Action::Add {
            disk: "/dev/vda".into(),
            role,
            start,
            end,
            type_hint: if swap {
                core::TypeHint::LinuxSwap
            } else {
                core::TypeHint::LinuxData
            },
        },
    }
}
fn format(n: u8, label: &str, swap: bool) -> work::Operation {
    work::Operation::Format {
        partition: format!("/dev/vda{n}"),
        format: if swap {
            work::FormatChoice::Swap
        } else {
            work::FormatChoice::Ext4
        },
        label: label.into(),
        uuid: None,
    }
}
#[test]
fn authored_references_reach_real_goals_and_initial_worlds_are_fresh_and_unsolved() {
    assert_eq!(labs().len(), 3);
    for s in labs() {
        let d = def(s);
        d.validate().unwrap();
        assert!(d.initial.mbr_profile.is_some());
        assert!(d.lesson.concepts.len() >= 4);
        assert!(!engine::solved(&s.initial, &s.goal));
        let a = fresh(s);
        assert_eq!(
            serde_json::to_value(sim(&a)).unwrap(),
            serde_json::to_value(work::Session::new(d).unwrap()).unwrap()
        );
        assert!(
            sim(&a)
                .assignment_editor
                .drafts
                .values()
                .all(|d| d.name.is_empty()
                    && d.start.is_empty()
                    && d.end.is_empty()
                    && d.label.is_empty()
                    && d.uuid.is_empty()
                    && d.mbr_role == Some(Kind::Primary))
        );
        assert!(!s.objective.contains("Read the contract"));
        assert!(!s.objective.contains("fresh virtual workspace"));
        let _ = solve(s);
    }
}
#[test]
fn filling_examples_is_inert_separate_and_ungraded_and_adverse_work_requires_reset_replay() {
    for s in labs() {
        let d = def(s);
        let mut a = fresh(s);
        work::binding::set_view(&mut a, s, View::Example).unwrap();
        let original =
            serde_json::to_value((&sim(&a).current, &sim(&a).assignment_editor)).unwrap();
        for (i, step) in d.lesson.worked.iter().enumerate() {
            let before = sim(&a).example.clone();
            work::binding::insert(&mut a, s, i).unwrap();
            assert_eq!(sim(&a).example, before, "Fill executed an action");
            assert_eq!(
                serde_json::to_value((&sim(&a).current, &sim(&a).assignment_editor)).unwrap(),
                original
            );
            work::binding::apply(&mut a, s, &step.operation, true).unwrap();
        }
        work::binding::check(&mut a, s, true).unwrap();
        assert!(d.practice_checks(sim(&a)).iter().all(|r| r.0));
        assert!(!progress::current_solved(&a, s));
        work::binding::apply(&mut a, s, &d.lesson.experiment, true).unwrap();
        assert!(d.practice_checks(sim(&a)).iter().any(|r| !r.0));
        work::binding::reset_example(&mut a, s).unwrap();
        for step in &d.lesson.worked {
            work::binding::apply(&mut a, s, &step.operation, true).unwrap();
        }
        assert!(d.practice_checks(sim(&a)).iter().all(|r| r.0));
        assert_eq!(
            serde_json::to_value((&sim(&a).current, &sim(&a).assignment_editor)).unwrap(),
            original
        );
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn alternate_positions_sizes_and_orders_pass_including_existing_supplied_identity() {
    let variants = [
        vec![
            work::Operation::Mbr {
                operation: Action::Create {
                    disk: "/dev/vda".into(),
                },
            },
            add(Kind::Primary, 4096, 12287, true),
            format(1, "scratch", true),
            add(Kind::Primary, 16384, 40959, false),
            format(2, "reports", false),
        ],
        vec![
            add(Kind::Extended, 32768, 122879, false),
            add(Kind::Logical, 34816, 43007, true),
            format(5, "scratch", true),
            add(Kind::Logical, 45056, 69631, false),
            format(6, "reports", false),
        ],
        vec![
            add(Kind::Logical, 65536, 73727, true),
            format(7, "ready_swap", true),
            format(6, "deliver", false),
        ],
    ];
    for (s, ops) in labs().into_iter().zip(variants) {
        let mut a = fresh(s);
        for op in ops {
            work::binding::apply(&mut a, s, &op, false).unwrap();
        }
        work::binding::check(&mut a, s, false).unwrap();
        assert!(
            progress::current_solved(&a, s),
            "{} {:?}",
            s.id,
            def(s).checks(sim(&a))
        );
    }
}
#[test]
fn forged_feedback_partial_work_and_extra_allocations_do_not_retain_success() {
    for (i, s) in labs().into_iter().enumerate() {
        let d = def(s);
        let mut a = fresh(s);
        if let State::PartitionFs { simulation } = &mut a.state {
            simulation.checked = true;
            simulation.transcript.push(work::Row {
                input: "check".into(),
                output: "all passed".into(),
            });
        }
        assert!(!progress::current_solved(&a, s));
        for op in d.reference.iter().take(d.reference.len() - 1) {
            work::binding::apply(&mut a, s, op, false).unwrap();
        }
        work::binding::check(&mut a, s, false).unwrap();
        assert!(!progress::current_solved(&a, s));
        assert!(d.checks(sim(&a)).iter().any(|r| !r.0));
        work::binding::apply(&mut a, s, d.reference.last().unwrap(), false).unwrap();
        assert!(progress::current_solved(&a, s));
        let extra = if i == 0 {
            add(Kind::Primary, 49152, 53247, false)
        } else {
            add(Kind::Logical, 81920, 86015, false)
        };
        work::binding::apply(&mut a, s, &extra, false).unwrap();
        assert!(!sim(&a).checked);
        assert!(!progress::current_solved(&a, s));
        assert!(!d.checks(sim(&a)).last().unwrap().0);
    }
    let s = labs()[2];
    let mut a = fresh(s);
    let before = serde_json::to_value(&a).unwrap();
    assert!(work::binding::apply(&mut a, s, &format(5, "overwritten", false), false).is_err());
    assert_eq!(serde_json::to_value(&a).unwrap(), before);
    let s = labs()[1];
    let mut a = solve(s);
    let before = serde_json::to_value(&a).unwrap();
    assert!(work::binding::apply(&mut a, s, &format(4, "container", false), false).is_err());
    assert_eq!(serde_json::to_value(&a).unwrap(), before);
    assert!(progress::current_solved(&a, s));
}
#[test]
fn safe_save_load_export_and_inspect_preserve_partial_work_separate_drafts_and_unrelated_attempt() {
    let s = labs()[2];
    let mut p = Progress {
        schema_version: 1,
        clock: 8,
        ..Default::default()
    };
    let old = &all()[0];
    p.attempts.insert(old.id.clone(), Attempt::new(old, 8));
    let untouched = serde_json::to_value(&p.attempts[&old.id]).unwrap();
    let mut a = fresh(s);
    work::binding::set_view(&mut a, s, View::Assignment).unwrap();
    work::binding::apply(&mut a, s, &def(s).reference[0], false).unwrap();
    let mut draft = sim(&a).assignment_editor.drafts["/dev/vda"].clone();
    draft.start = "unfinished".into();
    draft.mbr_role = Some(Kind::Logical);
    work::binding::set_draft(&mut a, s, "/dev/vda", draft, false).unwrap();
    work::binding::insert(&mut a, s, 1).unwrap();
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let original = serde_json::to_value(&p.attempts).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, all()).unwrap();
    let loaded = progress::load(&path, all()).unwrap();
    assert_eq!(serde_json::to_value(&loaded.attempts).unwrap(), original);
    assert_eq!(loaded.favorites, p.favorites);
    assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&old.id]).unwrap(),
        untouched
    );
    let export = dir.path().join("portable.json");
    backup::export(&export, &loaded, all()).unwrap();
    let inspected = backup::inspect(
        &export,
        all(),
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&inspected.progress.attempts).unwrap(),
        original
    );
    let mut resumed = loaded.attempts[&s.id].clone();
    for op in def(s).reference.iter().skip(1) {
        work::binding::apply(&mut resumed, s, op, false).unwrap();
    }
    work::binding::check(&mut resumed, s, false).unwrap();
    assert!(progress::current_solved(&resumed, s));
    assert_eq!(
        sim(&resumed).assignment_editor.drafts["/dev/vda"].start,
        "unfinished"
    );
}
#[test]
fn invalid_schema_and_unreachable_supplied_slot_are_not_authored_coverage() {
    let mut data: Value =
        serde_json::from_str(include_str!("../scenarios/mbr-layout-lessons-v1.json")).unwrap();
    data["schema_version"] = json!(2);
    assert!(
        pack::load(&data.to_string())
            .unwrap_err()
            .contains("schema")
    );
    let s = labs()[2];
    let mut d = def(s).clone();
    d.plan.mbr.as_mut().unwrap().targets[0].supplied_slot = Some(7);
    assert!(d.validate().is_err());
    let mut d = def(s).clone();
    d.lesson.worked.clear();
    assert!(d.validate().is_err());
    assert_eq!(
        pack::accepted323().unwrap().len(),
        323,
        "Frozen original catalog changed"
    );
}
