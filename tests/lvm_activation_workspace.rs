//! Synthetic binding/goal proof; no authored lesson or native acceptance claim.
use dlms_learning_lab::{lvm, lvm_workspace as work};
use serde_json::{Value, json};
fn original() -> work::Definition {
    let p: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-retirement-lessons-v1.json")).unwrap();
    serde_json::from_value(p["scenarios"][0]["goal"]["definition"].clone()).unwrap()
}
fn fixture() -> work::Definition {
    let mut raw = serde_json::to_value(original()).unwrap();
    raw["retirement"] = Value::Null;
    raw["lesson"]["retirement"] = Value::Null;
    for example in [false, true] {
        let world = if example {
            &raw["lesson"]["fixture"]
        } else {
            &raw["initial"]
        };
        let mut s: lvm::State = serde_json::from_value(world.clone()).unwrap();
        let name = s.groups.keys().next().unwrap().clone();
        s.activation = Some(lvm::activation::Model {
            active: s
                .groups
                .iter()
                .flat_map(|(g, d)| d.volumes.keys().map(move |v| (format!("{g}/{v}"), false)))
                .collect(),
            open: vec![],
        });
        let plan = json!({"groups":[{"name":name,"active":true}]});
        if example {
            raw["lesson"]["fixture"] = serde_json::to_value(s).unwrap();
            raw["lesson"]["availability"] = plan;
        } else {
            raw["initial"] = serde_json::to_value(s).unwrap();
            raw["availability"] = plan;
        }
    }
    raw["lesson"]["worked"] = json!([{"text":"lvs","explanation":"Inspect metadata and the explicit simulated availability column."},{"text":"vgchange -ay workshop","explanation":"Make the supplied workshop LVs available without changing their allocations or files."},{"text":"lvs","explanation":"Observe current available LVs and retained original data."}]);
    raw["lesson"]["experiment"] = json!("vgchange -an workshop");
    raw["lesson"]["repair"] = json!("vgchange -ay workshop");
    serde_json::from_value(raw).unwrap()
}
fn all(rows: &[(bool, String)]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.0)
}
#[test]
fn current_availability_and_readable_original_data_determine_success() {
    let d = fixture();
    d.validate().unwrap();
    let mut s = work::Session::new(&d).unwrap();
    let rows = d.checks(&s);
    assert!(!all(&rows));
    assert!(rows.iter().any(|r| r.0 && r.1.contains("unchanged")));
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("unavailable")));
    s.checked = true;
    s.transcript.push(work::Row {
        input: "lvs".into(),
        output: "all active".into(),
    });
    assert!(!all(&d.checks(&s)));
    s.execute(&d, "vgchange -a y records", false).unwrap();
    assert!(all(&d.checks(&s)));
    assert_eq!(s.current.groups, d.initial.groups);
    assert_eq!(s.current.devices, d.initial.devices);
}
#[test]
fn deactivation_breaks_current_success_and_equivalent_reactivation_repairs_it() {
    let d = fixture();
    let mut s = work::Session::new(&d).unwrap();
    s.execute(&d, "vgchange --activate y records", false)
        .unwrap();
    s.checked = true;
    assert!(all(&d.checks(&s)));
    s.execute(&d, "vgchange -an records", false).unwrap();
    assert!(!s.checked);
    assert!(!all(&d.checks(&s)));
    s.execute(&d, "vgchange -ay records", false).unwrap();
    assert!(all(&d.checks(&s)));
}
#[test]
fn partial_activation_missing_volumes_and_changed_files_do_not_false_pass() {
    let d = fixture();
    let mut s = work::Session::new(&d).unwrap();
    s.current
        .activation
        .as_mut()
        .unwrap()
        .active
        .insert("records/ledger".into(), true);
    let rows = d.checks(&s);
    assert!(!all(&rows));
    assert!(
        rows.iter()
            .any(|r| r.0 && r.1.contains("reader records/ledger"))
    );
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("active")));
    s.execute(&d, "vgchange -ay records", false).unwrap();
    s.execute(&d, "lvremove -y records/scratch", false).unwrap();
    assert!(!all(&d.checks(&s)));
    assert!(d.checks(&s).iter().any(|r| !r.0 && r.1.contains("active")));
    let mut s = work::Session::new(&d).unwrap();
    s.execute(&d, "vgchange -ay records", false).unwrap();
    s.current
        .groups
        .get_mut("records")
        .unwrap()
        .volumes
        .get_mut("ledger")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .get_mut("/keep.txt")
        .unwrap()
        .contents = "damaged".into();
    assert!(
        d.checks(&s)
            .iter()
            .any(|r| !r.0 && r.1.contains("readable, but"))
    );
    assert!(!all(&d.checks(&s)));
}
#[test]
fn independent_drafts_practice_experiment_and_roundtrip_keep_their_own_state() {
    let d = fixture();
    let mut s = work::Session::new(&d).unwrap();
    s.set_command(&d, "independent draft".into(), false)
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
    let copy: work::Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    d.bound(&copy).unwrap();
    assert_eq!(copy, s);
    s.reset_example(&d).unwrap();
    assert_eq!(s.example, d.lesson.fixture);
    assert_eq!(s.current, d.initial);
    assert_eq!(s.command, "independent draft");
}
#[test]
fn exclusive_goal_forms_profiles_missing_groups_duplicates_and_busy_targets_are_rejected() {
    let d = fixture();
    for mode in 0..7 {
        let mut raw = serde_json::to_value(&d).unwrap();
        match mode {
            0 => raw["availability"]["groups"][0]["name"] = json!("absent"),
            1 => {
                raw["availability"]["groups"] =
                    json!([{"name":"records","active":true},{"name":"records","active":false}])
            }
            2 => raw["availability"] = Value::Null,
            3 => raw["initial"]["activation"] = Value::Null,
            4 => {
                raw["retirement"] = json!({"retired_volumes":[],"retired_groups":[],"cleared_pvs":["/dev/recordspare"]})
            }
            5 => {
                raw["initial"]["activation"]["active"]["records/peer"] = json!(true);
                raw["initial"]["activation"]["open"] = json!(["records/peer"]);
                raw["availability"]["groups"][0]["active"] = json!(false);
            }
            _ => raw["lesson"]["availability"] = Value::Null,
        }
        let bad: work::Definition = serde_json::from_value(raw).unwrap();
        assert!(bad.shape().is_err(), "mode{mode}");
    }
    let mut raw = serde_json::to_value(&d).unwrap();
    raw["availability"]["groups"] = json!(vec![json!({"name":"records","active":true}); 5]);
    assert!(serde_json::from_value::<work::Definition>(raw).is_err());
}
#[test]
fn old_none_fields_and_immutable_open_facts_cannot_gain_new_goal_credit() {
    let old = original();
    old.validate().unwrap();
    let oldjson = serde_json::to_value(&old).unwrap();
    assert!(oldjson.get("availability").is_none());
    assert!(oldjson["initial"].get("activation").is_none());
    let mut s = work::Session::new(&old).unwrap();
    s.current = fixture().initial;
    assert!(old.bound(&s).is_err());
    let d = fixture();
    let mut s = work::Session::new(&d).unwrap();
    s.current.activation = None;
    assert!(d.bound(&s).is_err());
    let mut s = work::Session::new(&d).unwrap();
    s.current
        .activation
        .as_mut()
        .unwrap()
        .active
        .insert("records/peer".into(), true);
    s.current
        .activation
        .as_mut()
        .unwrap()
        .open
        .push("records/peer".into());
    assert!(d.bound(&s).is_err());
}
