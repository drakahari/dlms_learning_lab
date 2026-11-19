//! Opt-in outcome/binding proof over a synthetic, unregistered teaching fixture.
use dlms_learning_lab::{lvm, lvm_workspace as work};
use serde_json::{Value, json};
fn old() -> work::Definition {
    let raw: Value = serde_json::from_str(include_str!("../scenarios/lvm-growth-v1.json")).unwrap();
    serde_json::from_value(raw["scenarios"][0]["goal"]["definition"].clone()).unwrap()
}
fn fixture() -> work::Definition {
    let mut raw = serde_json::to_value(old()).unwrap();
    for path in ["initial", "example"] {
        let world = if path == "initial" {
            &mut raw["initial"]
        } else {
            &mut raw["lesson"]["fixture"]
        };
        world["provisioning"] = json!("Linear");
        let state: lvm::State = serde_json::from_value(world.clone()).unwrap();
        let g = state.groups.keys().next().unwrap();
        let data = &state.groups[g];
        let devices = state
            .devices
            .iter()
            .filter(|(_, d)| d.group == *g)
            .map(|(p, _)| p.clone())
            .collect::<Vec<_>>();
        let plan = json!({"groups":[{"name":g,"extent_bytes":data.extent_bytes,"devices":devices,"reserve_extents":state.free_extents(g).unwrap()-2}],"volumes":[{"id":format!("{g}/trial"),"bytes":data.extent_bytes*2}]});
        if path == "initial" {
            raw["targets"] = json!([]);
            raw["plan"] = plan;
        } else {
            raw["lesson"]["targets"] = json!([]);
            raw["lesson"]["plan"] = plan;
        }
    }
    raw["lesson"]["worked"] = json!([
        {"text":"vgs","explanation":"Inspect the separate pool."},
        {"text":"lvcreate -l 2 -n trial aspen","explanation":"Allocate only the new raw example."},
        {"text":"lvs","explanation":"Verify both existing data and the new allocation."}]);
    raw["lesson"]["experiment"] = json!("lvremove -y aspen/trial");
    raw["lesson"]["repair"] = json!("lvcreate -l 2 -n trial aspen");
    serde_json::from_value(raw).unwrap()
}
fn passed(rows: &[(bool, String)]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.0)
}
#[test]
fn complete_partial_and_wrong_allocations_are_graded_from_actual_state() {
    let d = fixture();
    d.validate().unwrap();
    let mut s = work::Session::new(&d).unwrap();
    let rows = d.checks(&s);
    assert_eq!(rows.len(), 6);
    assert!(!passed(&rows));
    assert!(rows.iter().any(|r| r.0));
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("volume missing")));
    s.execute(&d, "lvcreate -l 1 -n trial records", false)
        .unwrap();
    let rows = d.checks(&s);
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("4194304 bytes")));
    s.execute(&d, "lvremove -y records/trial", false).unwrap();
    s.execute(&d, "lvcreate --name trial --size 8M records", false)
        .unwrap();
    assert!(passed(&d.checks(&s)));
    assert!(
        s.current
            .volume("records/trial")
            .unwrap()
            .1
            .filesystem
            .is_none()
    );
}
#[test]
fn equivalent_creation_and_removal_orders_invalidate_and_recover_current_success() {
    let d = fixture();
    let mut s = work::Session::new(&d).unwrap();
    s.execute(&d, "lvcreate -L 8M -n trial records", false)
        .unwrap();
    assert!(passed(&d.checks(&s)));
    s.checked = true;
    s.execute(&d, "lvcreate -l 1 -n unintended records", false)
        .unwrap();
    assert!(!s.checked);
    assert!(!passed(&d.checks(&s)));
    let rows = d.checks(&s);
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("free extents")));
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("only the requested"))
    );
    s.execute(&d, "lvremove -y records/unintended", false)
        .unwrap();
    assert!(passed(&d.checks(&s)));
    s.execute(&d, "lvremove --yes /dev/records/trial", false)
        .unwrap();
    assert!(!passed(&d.checks(&s)));
    s.execute(&d, "lvcreate -l 2 --name trial records", false)
        .unwrap();
    assert!(passed(&d.checks(&s)));
}
#[test]
fn worked_insertion_running_and_reset_never_touch_assignment_or_drafts() {
    let d = fixture();
    let mut s = work::Session::new(&d).unwrap();
    let assignment = s.current.clone();
    s.set_command(&d, "my independent draft".into(), false)
        .unwrap();
    s.insert(&d, 1).unwrap();
    assert_eq!(s.example, s.example_initial);
    assert_eq!(s.current, assignment);
    for command in &d.lesson.worked {
        s.execute(&d, &command.text, true).unwrap();
    }
    assert!(passed(&d.practice_checks(&s)));
    assert!(!passed(&d.checks(&s)));
    assert_eq!(s.current, assignment);
    s.execute(&d, &d.lesson.experiment, true).unwrap();
    assert!(!passed(&d.practice_checks(&s)));
    s.execute(&d, &d.lesson.repair, true).unwrap();
    assert!(passed(&d.practice_checks(&s)));
    s.reset_example(&d).unwrap();
    assert_eq!(s.example, s.example_initial);
    assert_eq!(s.current, assignment);
    assert_eq!(s.command, "my independent draft");
}
#[test]
fn immutable_profile_device_context_and_old_absent_fields_cannot_be_forged() {
    let old = old();
    old.validate().unwrap();
    let mut s = work::Session::new(&old).unwrap();
    let wire = serde_json::to_value(&old).unwrap();
    assert!(wire.get("plan").is_none());
    assert!(wire["lesson"].get("plan").is_none());
    s.current.provisioning = Some(lvm::provision::Profile::Linear);
    assert!(old.bound(&s).is_err());
    let d = fixture();
    let mut s = work::Session::new(&d).unwrap();
    s.current.provisioning = None;
    assert!(d.bound(&s).is_err());
    let mut s = work::Session::new(&d).unwrap();
    s.current.devices.get_mut("/dev/recordsa").unwrap().bytes += lvm::MIB;
    assert!(d.bound(&s).is_err());
    let mut s = work::Session::new(&d).unwrap();
    let before = serde_json::to_value(&s).unwrap();
    assert!(s.execute(&d, "lvremove -y records/reports", false).is_err());
    assert_eq!(serde_json::to_value(s).unwrap(), before);
}
#[test]
fn malformed_unreachable_or_cross_example_plans_are_honestly_refused() {
    let d = fixture();
    let good = serde_json::to_value(&d).unwrap();
    for change in [0, 1, 2, 3, 4, 5] {
        let mut raw = good.clone();
        match change {
            0 => raw["plan"]["groups"][0]["devices"] = json!(["/dev/recordsa", "/dev/recordsa"]),
            1 => raw["plan"]["volumes"][0]["bytes"] = json!(1),
            2 => raw["plan"]["volumes"][0]["id"] = json!("records/reports"),
            3 => raw["lesson"]["plan"] = Value::Null,
            4 => raw["lesson"]["plan"]["groups"][0]["name"] = json!("records"),
            _ => raw["lesson"]["plan"]["volumes"][0]["bytes"] = json!(128 * 64 * lvm::MIB),
        }
        let d: work::Definition = serde_json::from_value(raw).unwrap();
        assert!(d.validate().is_err(), "change{change}");
    }
    let mut raw = good;
    raw["plan"]["groups"][0]["devices"][0] = json!("x".repeat(65));
    assert!(serde_json::from_value::<work::Definition>(raw).is_err());
}
