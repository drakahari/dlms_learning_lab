//! Synthetic typed outcome/binding proof. No authored-lab or native acceptance.
use dlms_learning_lab::{
    lvm::{self, DeviceRole, MIB},
    lvm_workspace as work,
};
use serde_json::{Value, json};
fn original() -> work::Definition {
    let p: Value = serde_json::from_str(include_str!("../scenarios/lvm-growth-v1.json")).unwrap();
    serde_json::from_value(p["scenarios"][0]["goal"]["definition"].clone()).unwrap()
}
fn setup(s: &mut lvm::State, t: &mut work::Target, minimum: u64) {
    s.pv_capacity = Some(lvm::pv_capacity::Profile::MetadataSize);
    for d in s.devices.values_mut() {
        if d.role == DeviceRole::PhysicalVolume {
            d.pv_bytes = Some(d.bytes);
        }
    }
    let group = t.volume.split_once('/').unwrap().0;
    let path = s
        .devices
        .iter()
        .find(|(_, d)| d.group == group)
        .unwrap()
        .0
        .clone();
    s.devices.get_mut(&path).unwrap().pv_bytes = Some(17 * MIB);
    s.devices.get_mut(&path).unwrap().bytes = 65 * MIB;
    t.minimum_lv = minimum;
    t.minimum_filesystem = minimum;
    t.fill_lv = true;
    t.pv_sizes = vec![work::pv_capacity::Target {
        device: path,
        bytes: 65 * MIB,
    }];
}
fn fixture() -> work::Definition {
    let mut d = original();
    setup(&mut d.initial, &mut d.targets[0], 24 * MIB);
    setup(&mut d.lesson.fixture, &mut d.lesson.targets[0], 16 * MIB);
    d.lesson.worked = vec![
        work::Command {
            text: "pvs".into(),
            explanation: "Read the currently recognized PV capacity.".into(),
        },
        work::Command {
            text: "pvresize /dev/aspena".into(),
            explanation:
                "Recognize the supplied larger device without changing the LV or filesystem.".into(),
        },
        work::Command {
            text: "lvextend -L 16M aspen/sample".into(),
            explanation: "Allocate the now-available physical extents to the intended LV.".into(),
        },
        work::Command {
            text: "resize2fs /dev/aspen/sample".into(),
            explanation: "Grow its separate filesystem after the LV, retaining original data."
                .into(),
        },
    ];
    d.lesson.experiment = "pvresize --setphysicalvolumesize 33M /dev/aspena".into();
    d.lesson.repair = "pvresize /dev/aspena".into();
    d
}
fn all(rows: &[(bool, String)]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.0)
}
fn solve(s: &mut work::Session, d: &work::Definition) {
    for c in [
        "pvresize /dev/recordsa",
        "lvextend -L 24M records/reports",
        "resize2fs /dev/records/reports",
    ] {
        s.execute(d, c, false).unwrap();
    }
}
#[test]
fn pv_then_lv_then_filesystem_each_have_current_outcomes_not_command_credit() {
    let d = fixture();
    d.validate().unwrap();
    let mut s = work::Session::new(&d).unwrap();
    s.checked = true;
    s.transcript.push(work::Row {
        input: "pvs".into(),
        output: "all complete".into(),
    });
    assert!(!all(&d.checks(&s)));
    let before = s.clone();
    assert!(
        s.execute(&d, "lvextend -L 24M records/reports", false)
            .is_err()
    );
    assert_eq!(s, before);
    s.execute(&d, "pvresize /dev/recordsa", false).unwrap();
    let rows = d.checks(&s);
    assert!(rows.iter().any(|r| r.0 && r.1.contains("recognized PV")));
    assert!(!all(&rows));
    s.execute(&d, "lvextend -L 24M records/reports", false)
        .unwrap();
    assert!(!all(&d.checks(&s)));
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("filesystem"))
    );
    s.execute(&d, "resize2fs /dev/records/reports", false)
        .unwrap();
    assert!(all(&d.checks(&s)));
}
#[test]
fn alternate_commands_pass_and_safe_metadata_change_invalidates_then_repairs() {
    let d = fixture();
    let mut a = work::Session::new(&d).unwrap();
    solve(&mut a, &d);
    let mut b = work::Session::new(&d).unwrap();
    for c in [
        "pvresize --setphysicalvolumesize 65m /dev/recordsa",
        "lvextend --extents 6 /dev/records/reports",
        "resize2fs /dev/records/reports 6144",
    ] {
        b.execute(&d, c, false).unwrap();
    }
    assert_eq!(a.current, b.current);
    assert!(all(&d.checks(&b)));
    a.checked = true;
    a.execute(
        &d,
        "pvresize --setphysicalvolumesize 33M /dev/recordsa",
        false,
    )
    .unwrap();
    assert!(!a.checked);
    assert!(!all(&d.checks(&a)));
    assert!(
        d.checks(&a)
            .iter()
            .any(|r| !r.0 && r.1.contains("recognized PV"))
    );
    a.execute(&d, "pvresize /dev/recordsa", false).unwrap();
    assert!(all(&d.checks(&a)));
}
#[test]
fn optional_preserved_volume_goal_does_not_allow_unnecessary_lv_or_filesystem_changes() {
    let mut d = fixture();
    d.initial.devices.get_mut("/dev/recordsa").unwrap().pv_bytes = Some(33 * MIB);
    d.targets[0].minimum_lv = 8 * MIB;
    d.targets[0].minimum_filesystem = 8 * MIB;
    d.targets[0].preserve_volume = true;
    d.targets[0].pv_sizes[0].bytes = 25 * MIB;
    let mut s = work::Session::new(&d).unwrap();
    assert!(!all(&d.checks(&s)));
    s.execute(
        &d,
        "pvresize --setphysicalvolumesize 25M /dev/recordsa",
        false,
    )
    .unwrap();
    assert!(all(&d.checks(&s)));
    s.execute(&d, "lvextend -l +1 records/reports", false)
        .unwrap();
    s.execute(&d, "resize2fs /dev/records/reports", false)
        .unwrap();
    let rows = d.checks(&s);
    assert!(!all(&rows));
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("preserve the original LV"))
    );
}
#[test]
fn unrelated_pv_metadata_and_changed_original_files_fail_without_rejecting_retry_context() {
    let mut d = fixture();
    d.initial.devices.insert(
        "/dev/orphan".into(),
        lvm::Device {
            bytes: 33 * MIB,
            role: DeviceRole::PhysicalVolume,
            pv_bytes: Some(33 * MIB),
            group: String::new(),
        },
    );
    d.validate().unwrap();
    let mut s = work::Session::new(&d).unwrap();
    solve(&mut s, &d);
    s.execute(
        &d,
        "pvresize --setphysicalvolumesize 17M /dev/orphan",
        false,
    )
    .unwrap();
    d.bound(&s).unwrap();
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("Unrelated /dev/orphan"))
    );
    s.execute(&d, "pvresize /dev/orphan", false).unwrap();
    assert!(all(&d.checks(&s)));
    s.current
        .groups
        .get_mut("records")
        .unwrap()
        .volumes
        .get_mut("reports")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .values_mut()
        .next()
        .unwrap()
        .contents = "damaged".into();
    d.bound(&s).unwrap();
    assert!(!all(&d.checks(&s)));
}
#[test]
fn separate_examples_drafts_reset_and_roundtrip_keep_assignment_state() {
    let d = fixture();
    let mut s = work::Session::new(&d).unwrap();
    s.set_command(&d, "unfinished assignment".into(), false)
        .unwrap();
    s.insert(&d, 1).unwrap();
    assert_eq!(s.current, d.initial);
    assert_eq!(s.example, d.lesson.fixture);
    for c in &d.lesson.worked {
        s.execute(&d, &c.text, true).unwrap();
    }
    assert!(all(&d.practice_checks(&s)));
    assert!(!all(&d.checks(&s)));
    s.execute(&d, &d.lesson.experiment, true).unwrap();
    assert!(!all(&d.practice_checks(&s)));
    s.execute(&d, &d.lesson.repair, true).unwrap();
    assert!(all(&d.practice_checks(&s)));
    solve(&mut s, &d);
    let copy: work::Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    d.bound(&copy).unwrap();
    assert_eq!(copy, s);
    assert!(all(&d.checks(&copy)));
    s.reset_example(&d).unwrap();
    assert!(all(&d.checks(&s)));
    assert_eq!(s.example, d.lesson.fixture);
    assert_eq!(s.command, "unfinished assignment");
    assert!(!all(&d.checks(&work::Session::new(&d).unwrap())));
}
#[test]
fn invalid_or_unreachable_pv_targets_and_old_goal_injection_are_rejected() {
    let d = fixture();
    let raw = serde_json::to_value(&d).unwrap();
    for mode in 0..8 {
        let mut bad = raw.clone();
        match mode {
            0 => bad["targets"][0]["pv_sizes"][0]["device"] = json!("/dev/absent"),
            1 => bad["targets"][0]["pv_sizes"][0]["bytes"] = json!(13 * MIB),
            2 => bad["targets"][0]["pv_sizes"][0]["bytes"] = json!(66 * MIB),
            3 => bad["targets"][0]["pv_sizes"][0]["bytes"] = json!(33 * MIB + 1),
            4 => bad["targets"][0]["pv_sizes"]
                .as_array_mut()
                .unwrap()
                .push(raw["targets"][0]["pv_sizes"][0].clone()),
            5 => bad["targets"][0]["pv_sizes"] = json!([]),
            6 => bad["targets"][0]["preserve_volume"] = json!(true),
            _ => bad["targets"][0]["pv_sizes"][0]["bytes"] = json!(33 * MIB + 512),
        }
        assert!(
            serde_json::from_value::<work::Definition>(bad)
                .unwrap()
                .shape()
                .is_err(),
            "mode{mode}"
        );
    }
    let mut bad = raw.clone();
    bad["targets"][0]["pv_sizes"] = json!(vec![raw["targets"][0]["pv_sizes"][0].clone(); 9]);
    assert!(serde_json::from_value::<work::Definition>(bad).is_err());
    let old = original();
    let mut bad = serde_json::to_value(&old).unwrap();
    bad["targets"][0]["pv_sizes"] = raw["targets"][0]["pv_sizes"].clone();
    assert!(
        serde_json::from_value::<work::Definition>(bad)
            .unwrap()
            .shape()
            .is_err()
    );
    let mut s = work::Session::new(&old).unwrap();
    s.current = d.initial;
    assert!(old.bound(&s).is_err());
}
