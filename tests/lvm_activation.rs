//! Optional virtual availability only; no host LVM, filesystem or process access.
use dlms_learning_lab::{
    lvm::{self, ErrorKind, activation},
    lvm_workspace as work,
};
use serde_json::{Value, json};
fn old() -> work::Definition {
    let p: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-retirement-lessons-v1.json")).unwrap();
    serde_json::from_value(p["scenarios"][0]["goal"]["definition"].clone()).unwrap()
}
fn fixture(open: bool) -> lvm::State {
    let mut s = old().initial;
    s.activation = Some(activation::Model {
        active: s
            .groups
            .iter()
            .flat_map(|(g, d)| {
                d.volumes.keys().map(move |v| {
                    (
                        format!("{g}/{v}"),
                        !matches!(v.as_str(), "ledger" | "scratch"),
                    )
                })
            })
            .collect(),
        open: if open {
            vec!["records/peer".into()]
        } else {
            vec![]
        },
    });
    s.validate().unwrap();
    s
}
fn refused(s: &mut lvm::State, c: &str) -> lvm::Error {
    let before = s.clone();
    let e = s.execute(c).expect_err(c);
    assert_eq!(*s, before, "{c}");
    e
}
#[test]
fn availability_is_separate_from_allocation_and_retained_data() {
    let mut s = fixture(false);
    let groups = s.groups.clone();
    let devices = s.devices.clone();
    let free = s.free_extents("records").unwrap();
    let e = s.read_file("records/ledger", "/keep.txt").unwrap_err();
    assert!(e.message.contains("inactive"));
    assert_eq!(
        refused(&mut s, "resize2fs /dev/records/ledger").kind,
        ErrorKind::Occupied
    );
    s.execute("vgchange -a y records").unwrap();
    assert!(
        s.read_file("records/ledger", "/keep.txt")
            .unwrap()
            .contains("records/ledger")
    );
    assert!(activation::is_active(&s, "/dev/records/ledger").unwrap());
    s.execute("vgchange --activate n records").unwrap();
    assert!(!activation::is_active(&s, "records/ledger").unwrap());
    assert!(s.read_file("records/ledger", "/keep.txt").is_err());
    assert_eq!(s.groups, groups);
    assert_eq!(s.devices, devices);
    assert_eq!(s.free_extents("records").unwrap(), free);
    s.execute("vgchange -ay records").unwrap();
    s.execute("resize2fs /dev/records/ledger").unwrap();
    assert_eq!(s.groups, groups);
}
#[test]
fn common_short_long_and_combined_activation_forms_are_equivalent() {
    let mut results = Vec::new();
    for c in [
        "vgchange -a y records",
        "vgchange --activate y records",
        "vgchange -ay records",
    ] {
        let mut s = fixture(true);
        s.execute(c).unwrap();
        results.push(s);
    }
    assert!(results.windows(2).all(|a| a[0] == a[1]));
    for c in [
        "vgchange -a n records",
        "vgchange --activate n records",
        "vgchange -an records",
    ] {
        let mut s = fixture(false);
        s.execute(c).unwrap();
        assert!(s.activation.unwrap().active.values().all(|v| !*v));
    }
}
#[test]
fn supplied_open_consumer_refuses_deactivation_without_partial_changes() {
    let mut s = fixture(true);
    for c in ["vgchange -an records", "lvremove -y records/peer"] {
        let e = refused(&mut s, c);
        assert_eq!(e.kind, ErrorKind::Occupied);
        assert!(e.message.contains("open"));
    }
    assert!(activation::is_active(&s, "records/peer").unwrap());
    assert!(!activation::is_active(&s, "records/ledger").unwrap());
    s.execute("vgchange -ay records").unwrap();
    assert!(activation::is_active(&s, "records/ledger").unwrap());
}
#[test]
fn current_reports_are_labeled_and_reflect_partial_active_state() {
    let mut s = fixture(false);
    let out = s.execute("lvs").unwrap();
    assert!(out.starts_with("LV VG LSize(B) #LE Type ModelActive\n"));
    assert!(out.contains("ledger records 8388608 2 linear inactive"));
    assert!(out.contains("peer records 8388608 2 linear active"));
    s.execute("vgchange -an records").unwrap();
    assert!(
        !s.execute("lvs")
            .unwrap()
            .lines()
            .skip(1)
            .any(|line| line.ends_with(" active"))
    );
}
#[test]
fn creation_removal_and_group_retirement_keep_availability_inventory_exact() {
    let mut s = fixture(false);
    s.execute("lvremove -y records/scratch").unwrap();
    assert!(
        !s.activation
            .as_ref()
            .unwrap()
            .active
            .contains_key("records/scratch")
    );
    s.execute("lvcreate -l 1 -n trial records").unwrap();
    assert!(activation::is_active(&s, "records/trial").unwrap());
    s.execute("vgcreate staging /dev/recordspare").unwrap();
    s.execute("lvcreate -l 1 -n raw staging").unwrap();
    assert!(activation::is_active(&s, "staging/raw").unwrap());
    s.execute("vgchange -an staging").unwrap();
    s.execute("lvremove -y staging/raw").unwrap();
    s.execute("vgremove staging").unwrap();
    s.execute("pvremove /dev/recordspare").unwrap();
    s.validate().unwrap();
    assert!(
        !s.activation
            .as_ref()
            .unwrap()
            .active
            .keys()
            .any(|id| id.starts_with("staging/"))
    );
    assert_eq!(
        lvm::State::from_json(&serde_json::to_vec(&s).unwrap()).unwrap(),
        s
    );
}
#[test]
fn unsupported_syntax_flags_missing_groups_and_shell_escapes_fail_honestly() {
    let mut s = fixture(false);
    for c in [
        "vgchange",
        "vgchange -ay",
        "vgchange -a ay records",
        "vgchange -a yes records",
        "vgchange --activationmode partial records",
        "vgchange -an --force records",
        "vgchange -a y records another",
        "vgchange -a y /dev/records",
        "vgchange -a y ../records",
        "vgchange -a y records; id",
        "vgchange -a y $(id)",
        "vgchange -a y records | cat",
    ] {
        refused(&mut s, c);
    }
    assert_eq!(
        refused(&mut s, "vgchange -ay absent").kind,
        ErrorKind::Unknown
    );
    assert_eq!(refused(&mut s, &"x".repeat(2049)).kind, ErrorKind::Limit);
    assert_eq!(
        refused(&mut s, "mount /dev/records/ledger /mnt").message,
        activation::HELP
    );
}
#[test]
fn malformed_imports_incomplete_keys_open_inactive_and_oversized_lists_are_rejected() {
    let raw = serde_json::to_value(fixture(false)).unwrap();
    for mode in 0..7 {
        let mut bad = raw.clone();
        match mode {
            0 => {
                bad["activation"]["active"]
                    .as_object_mut()
                    .unwrap()
                    .remove("records/cache");
            }
            1 => bad["activation"]["active"]["/dev/records/cache"] = json!(true),
            2 => bad["activation"]["open"] = json!(["records/ledger"]),
            3 => bad["activation"]["open"] = json!(["records/peer", "records/peer"]),
            4 => bad["activation"]["open"] = json!(vec!["records/peer"; 17]),
            5 => bad["activation"]["unknown"] = json!(true),
            _ => bad["provisioning"] = Value::Null,
        }
        assert!(
            lvm::State::from_json(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "mode{mode}"
        );
    }
    let mut bad = raw;
    bad["activation"]["active"]["x".repeat(65)] = json!(false);
    assert!(lvm::State::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
}
#[test]
fn old_absent_profiles_cannot_gain_availability_from_imported_current_work() {
    let d = old();
    let mut original = d.initial.clone();
    let value = serde_json::to_value(&original).unwrap();
    assert!(value.get("activation").is_none());
    assert!(activation::is_active(&original, "records/ledger").is_err());
    assert!(!original.execute("lvs").unwrap().contains("ModelActive"));
    assert!(original.read_file("records/ledger", "/keep.txt").is_ok());
    for c in [
        "vgchange -ay records",
        "vgchange -a y records",
        "vgchange --unsupported records",
    ] {
        let expected = original.command_help();
        let e = refused(&mut original, c);
        assert_eq!(e.kind, ErrorKind::Unsupported);
        assert_eq!(e.message, expected);
    }
    let mut s = work::Session::new(&d).unwrap();
    s.current = fixture(false);
    assert!(d.bound(&s).is_err());
    let mut a = fixture(true);
    let b = a.clone();
    a.activation.as_mut().unwrap().open.clear();
    assert!(!work::same_problem(&b, &a));
}
