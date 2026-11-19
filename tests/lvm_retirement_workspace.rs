//! Typed outcomes/source binding over private synthetic fixtures, not catalog credit.
use dlms_learning_lab::{
    lvm::{self, MIB, provision::Profile},
    lvm_workspace as work,
};
use serde_json::{Value, json};
fn original() -> work::Definition {
    let p: Value = serde_json::from_str(include_str!("../scenarios/lvm-growth-v1.json")).unwrap();
    serde_json::from_value(p["scenarios"][0]["goal"]["definition"].clone()).unwrap()
}
fn fixture(groups: bool) -> work::Definition {
    let mut d = serde_json::to_value(original()).unwrap();
    for example in [false, true] {
        let key = if example { "aspen" } else { "records" };
        let mut s: lvm::State = serde_json::from_value(if example {
            d["lesson"]["fixture"].clone()
        } else {
            d["initial"].clone()
        })
        .unwrap();
        s.provisioning = Some(Profile::Lifecycle);
        let group = if groups {
            let path = format!("/dev/{key}retired");
            s.devices.insert(
                path.clone(),
                lvm::Device {
                    pv_bytes: None,
                    bytes: 25 * MIB,
                    role: lvm::DeviceRole::Blank,
                    group: String::new(),
                },
            );
            s.execute(&format!("vgcreate unused{key} {path}")).unwrap();
            format!("unused{key}")
        } else {
            key.into()
        };
        s.execute(&format!("lvcreate -l 1 -n scratch {group}"))
            .unwrap();
        s.execute(&format!("lvcreate -l 1 -n buffer {group}"))
            .unwrap();
        let plan = json!({"retired_volumes":[format!("{group}/scratch"),format!("{group}/buffer")],"retired_groups":if groups{vec![group.clone()]}else{vec![]},"cleared_pvs":if groups{vec![format!("/dev/{key}retired")]}else{vec![]}});
        if example {
            d["lesson"]["fixture"] = serde_json::to_value(s).unwrap();
            d["lesson"]["targets"] = json!([]);
            d["lesson"]["retirement"] = plan;
            let mut commands = vec![
                json!({"text":format!("lvremove -y {group}/scratch"),"explanation":"Return the unused raw allocation to the same pool."}),
                json!({"text":format!("lvremove -y {group}/buffer"),"explanation":"Retire only the other supplied unused raw allocation."}),
            ];
            if groups {
                commands.push(json!({"text":format!("vgremove {group}"),"explanation":"An empty group can be retired; its PV labels remain."}));
                commands.push(json!({"text":format!("pvremove /dev/{key}retired"),"explanation":"Clear only this unowned PV label; this is not secure erasure."}));
                d["lesson"]["experiment"] = json!(format!("pvcreate /dev/{key}retired"));
                d["lesson"]["repair"] = json!(format!("pvremove /dev/{key}retired"));
            } else {
                d["lesson"]["experiment"] = json!(format!("lvcreate -l 1 -n scratch {group}"));
                d["lesson"]["repair"] = json!(format!("lvremove -y {group}/scratch"));
            }
            d["lesson"]["worked"] = json!(commands);
        } else {
            d["initial"] = serde_json::to_value(s).unwrap();
            d["targets"] = json!([]);
            d["retirement"] = plan;
        }
    }
    serde_json::from_value(d).unwrap()
}
fn all(rows: &[(bool, String)]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.0)
}
fn solve(d: &work::Definition, s: &mut work::Session, groups: bool) {
    for c in [
        "lvremove -y records/buffer",
        "lvremove --yes /dev/records/scratch",
    ] {
        if !groups {
            s.execute(d, c, false).unwrap();
        }
    }
    if groups {
        for c in [
            "lvremove -y unusedrecords/buffer",
            "lvremove -y unusedrecords/scratch",
            "vgremove --yes unusedrecords",
            "pvremove /dev/recordsretired",
        ] {
            s.execute(d, c, false).unwrap();
        }
    }
}
#[test]
fn both_outcome_plans_grade_partial_and_complete_actual_objects_not_history() {
    for groups in [false, true] {
        let d = fixture(groups);
        d.validate().unwrap();
        let mut s = work::Session::new(&d).unwrap();
        let rows = d.checks(&s);
        assert!(!all(&rows));
        assert!(rows.iter().any(|r| r.0 && r.1.contains("unrelated")));
        s.checked = true;
        s.transcript.push(work::Row {
            input: "pretend solution".into(),
            output: "all retired".into(),
        });
        assert!(!all(&d.checks(&s)));
        solve(&d, &mut s, groups);
        assert!(all(&d.checks(&s)));
        assert!(work::same_problem(&d.initial, &s.current));
    }
}
#[test]
fn empty_group_is_partial_until_labels_are_removed_and_missing_target_is_specific() {
    let d = fixture(true);
    let mut s = work::Session::new(&d).unwrap();
    for c in [
        "lvremove -y unusedrecords/scratch",
        "lvremove -y unusedrecords/buffer",
        "vgremove unusedrecords",
    ] {
        s.execute(&d, c, false).unwrap();
    }
    let rows = d.checks(&s);
    assert!(!all(&rows));
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("unowned labeled PV"))
    );
    assert!(rows.iter().any(|r| r.0 && r.1.contains("unrelated")));
    s.execute(&d, "pvremove /dev/recordsretired", false)
        .unwrap();
    assert!(all(&d.checks(&s)));
}
#[test]
fn supported_reintroduction_invalidates_current_truth_and_retry_recovers() {
    for groups in [false, true] {
        let d = fixture(groups);
        let mut s = work::Session::new(&d).unwrap();
        solve(&d, &mut s, groups);
        s.checked = true;
        let (c, repair) = if groups {
            (
                "pvcreate /dev/recordsretired",
                "pvremove /dev/recordsretired",
            )
        } else {
            (
                "lvcreate -l 1 -n scratch records",
                "lvremove -y records/scratch",
            )
        };
        s.execute(&d, c, false).unwrap();
        assert!(!s.checked);
        assert!(!all(&d.checks(&s)));
        s.execute(&d, repair, false).unwrap();
        assert!(all(&d.checks(&s)));
    }
}
#[test]
fn practice_drafts_reset_and_roundtrip_are_independent_of_assignment() {
    for groups in [false, true] {
        let d = fixture(groups);
        let mut s = work::Session::new(&d).unwrap();
        s.set_command(&d, "assignment draft".into(), false).unwrap();
        s.insert(&d, 0).unwrap();
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
        s.reset_example(&d).unwrap();
        assert_eq!(s.current, d.initial);
        assert_eq!(s.command, "assignment draft");
        let restored: work::Session =
            serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        d.bound(&restored).unwrap();
        assert_eq!(s, restored);
        assert_eq!(work::Session::new(&d).unwrap().current, d.initial);
    }
}
#[test]
fn invalid_plans_aliases_duplicates_and_mixed_task_forms_are_rejected() {
    let d = fixture(true);
    for key in ["retired_volumes", "retired_groups", "cleared_pvs"] {
        let mut raw = serde_json::to_value(&d).unwrap();
        let first = raw["retirement"][key][0].clone();
        raw["retirement"][key].as_array_mut().unwrap().push(first);
        assert!(
            serde_json::from_value::<work::Definition>(raw)
                .unwrap()
                .shape()
                .is_err()
        );
    }
    for bad in [
        json!({"retired_volumes":["records/reports"],"retired_groups":[],"cleared_pvs":[]}),
        json!({"retired_volumes":["/dev/unusedrecords/scratch"],"retired_groups":[],"cleared_pvs":[]}),
        json!({"retired_volumes":[],"retired_groups":["records"],"cleared_pvs":[]}),
        json!({"retired_volumes":[],"retired_groups":[],"cleared_pvs":["/dev/recordsa"]}),
        json!({"retired_volumes":[],"retired_groups":[],"cleared_pvs":[]}),
    ] {
        let mut raw = serde_json::to_value(&d).unwrap();
        raw["retirement"] = bad;
        assert!(
            serde_json::from_value::<work::Definition>(raw)
                .unwrap()
                .shape()
                .is_err()
        );
    }
    let mut raw = serde_json::to_value(&d).unwrap();
    raw["plan"] = json!({"groups":[],"volumes":[]});
    assert!(
        serde_json::from_value::<work::Definition>(raw)
            .unwrap()
            .shape()
            .is_err()
    );
    let mut raw = serde_json::to_value(&d).unwrap();
    raw["retirement"]["cleared_pvs"] = json!(vec!["/dev/recordsretired"; 9]);
    assert!(serde_json::from_value::<work::Definition>(raw).is_err());
}
#[test]
fn source_profile_inventory_data_and_extra_allocations_cannot_forge_success() {
    let d = fixture(false);
    let mut s = work::Session::new(&d).unwrap();
    solve(&d, &mut s, false);
    s.execute(&d, "lvcreate -l 1 -n unexpected records", false)
        .unwrap();
    assert!(!all(&d.checks(&s)));
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("unrelated"))
    );
    let mut s = work::Session::new(&d).unwrap();
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
        .get_mut("/report.txt")
        .unwrap()
        .contents = "damaged".into();
    d.bound(&s).unwrap();
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("unrelated"))
    );
    let mut s = work::Session::new(&d).unwrap();
    s.current.provisioning=Some(Profile::Linear);
    assert!(d.bound(&s).is_err());
    let mut s = work::Session::new(&d).unwrap();
    s.current.devices.get_mut("/dev/recordsa").unwrap().bytes += MIB;
    assert!(d.bound(&s).is_err());
    let old = original();
    assert!(
        serde_json::to_value(&old)
            .unwrap()
            .get("retirement")
            .is_none()
    );
    let mut s = work::Session::new(&old).unwrap();
    s.current.provisioning = Some(Profile::Lifecycle);
    assert!(old.bound(&s).is_err());
}
