//! Source-only virtual rename. These tests never invoke LVM or touch a device.
use dlms_learning_lab::{
    lvm::{self, ErrorKind, rename},
    lvm_workspace as work,
};
use serde_json::{Value, json};
fn old() -> lvm::State {
    let p: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-activation-lessons-v1.json")).unwrap();
    serde_json::from_value(p["scenarios"][1]["goal"]["definition"]["initial"].clone()).unwrap()
}
fn fixture() -> lvm::State {
    let mut s = old();
    let mut next = 1;
    let groups = s
        .groups
        .keys()
        .map(|g| {
            let id = next;
            next += 1;
            (g.clone(), id)
        })
        .collect();
    let volumes = s
        .groups
        .iter()
        .flat_map(|(g, d)| d.volumes.keys().map(move |v| format!("{g}/{v}")))
        .map(|v| {
            let id = next;
            next += 1;
            (v, id)
        })
        .collect();
    s.identity = Some(rename::Identity {
        groups,
        volumes,
        next,
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
fn lv_forms_keep_same_object_and_original_data_not_a_new_allocation() {
    let before = fixture();
    let mut results = Vec::new();
    for c in [
        "lvrename live ledger records",
        "lvrename live/ledger records",
        "lvrename /dev/live/ledger records",
    ] {
        let mut s = before.clone();
        s.execute(c).unwrap();
        assert_eq!(
            s.groups["live"].volumes["records"],
            before.groups["live"].volumes["ledger"]
        );
        assert_eq!(
            s.identity.as_ref().unwrap().volumes["live/records"],
            before.identity.as_ref().unwrap().volumes["live/ledger"]
        );
        assert_eq!(
            s.identity.as_ref().unwrap().next,
            before.identity.as_ref().unwrap().next
        );
        assert_eq!(s.devices, before.devices);
        assert!(s.volume("live/ledger").is_err());
        assert_eq!(
            s.read_file("live/records", "/keep.txt").unwrap(),
            before.read_file("live/ledger", "/keep.txt").unwrap()
        );
        assert_eq!(s.activation.as_ref().unwrap().open, vec!["live/records"]);
        assert!(work::same_problem(&before, &s));
        results.push(s);
    }
    assert!(results.windows(2).all(|v| v[0] == v[1]));
}
#[test]
fn group_rename_updates_membership_paths_and_consumers_without_touching_contents() {
    let before = fixture();
    let mut s = before.clone();
    s.execute("vgrename live production").unwrap();
    assert_eq!(s.groups["production"], before.groups["live"]);
    assert_eq!(s.devices["/dev/livea"].group, "production");
    assert_eq!(
        s.identity.as_ref().unwrap().groups["production"],
        before.identity.as_ref().unwrap().groups["live"]
    );
    assert_eq!(
        s.identity.as_ref().unwrap().volumes["production/ledger"],
        before.identity.as_ref().unwrap().volumes["live/ledger"]
    );
    assert_eq!(
        s.activation.as_ref().unwrap().open,
        vec!["production/ledger"]
    );
    assert!(s.volume("live/ledger").is_err());
    assert!(
        s.read_file("production/ledger", "/keep.txt")
            .unwrap()
            .starts_with("live/ledger:")
    );
    assert!(work::same_problem(&before, &s));
    assert!(
        refused(&mut s, "vgchange -an production")
            .message
            .contains("open")
    );
    assert!(
        refused(&mut s, "lvremove -y production/ledger")
            .message
            .contains("open")
    );
}
#[test]
fn either_namespace_order_is_equivalent_and_roundtrip_preserves_exact_original() {
    let original = fixture();
    let mut a = original.clone();
    let mut b = original.clone();
    for c in ["lvrename cold archive records", "vgrename cold warehouse"] {
        a.execute(c).unwrap();
    }
    for c in [
        "vgrename cold warehouse",
        "lvrename /dev/warehouse/archive records",
    ] {
        b.execute(c).unwrap();
    }
    assert_eq!(a, b);
    assert!(work::same_problem(&original, &a));
    assert!(
        a.read_file("warehouse/records", "/keep.txt")
            .unwrap_err()
            .message
            .contains("inactive")
    );
    a.execute("vgrename warehouse cold").unwrap();
    a.execute("lvrename cold records archive").unwrap();
    assert_eq!(a, original);
    let bytes = serde_json::to_vec(&b).unwrap();
    assert_eq!(lvm::State::from_json(&bytes).unwrap(), b);
}
#[test]
fn collisions_missing_sources_and_wrong_paths_are_atomic() {
    let mut s = fixture();
    s.execute("lvcreate -l 1 -n other live").unwrap();
    for c in [
        "vgrename live cold",
        "vgrename live live",
        "lvrename live ledger other",
        "lvrename live ledger ledger",
    ] {
        assert_eq!(refused(&mut s, c).kind, ErrorKind::Occupied);
    }
    for c in [
        "vgrename absent fresh",
        "lvrename absent/ledger records",
        "lvrename live/absent records",
    ] {
        assert_eq!(refused(&mut s, c).kind, ErrorKind::Unknown);
    }
    for c in [
        "lvrename live ledger ../escape",
        "lvrename /dev/live/ledger /dev/live/new",
        "vgrename live /dev/new",
        "lvrename --force live ledger new",
        "vgrename -f live new",
        "lvrename live/ledger snapshot",
        "lvrename live/ledger name_rmeta",
        "vgrename live a;id",
        "lvrename live/ledger $(id)",
        "vgrename live fresh extra",
        "lvrename live/ledger",
    ] {
        refused(&mut s, c);
    }
}
#[test]
fn creations_deletions_never_reuse_retired_identity_or_misrepresent_rename() {
    let original = fixture();
    let mut s = original.clone();
    let old_id = s.identity.as_ref().unwrap().volumes["staging/buffer"];
    s.execute("lvremove -y staging/buffer").unwrap();
    assert!(work::same_problem(&original, &s));
    s.execute("lvcreate -l 1 -n buffer staging").unwrap();
    let new_id = s.identity.as_ref().unwrap().volumes["staging/buffer"];
    assert!(new_id > old_id);
    assert!(new_id >= original.identity.as_ref().unwrap().next);
    assert_eq!(s.groups["staging"], original.groups["staging"]); // look-alike storage, different object
    s.execute("lvremove -y staging/buffer").unwrap();
    s.execute("vgremove staging").unwrap();
    s.execute("vgcreate staging /dev/stagea").unwrap();
    assert_ne!(
        s.identity.as_ref().unwrap().groups["staging"],
        original.identity.as_ref().unwrap().groups["staging"]
    );
    s.execute("lvcreate -l 1 -n buffer staging").unwrap();
    s.validate().unwrap();
    let wire = serde_json::to_vec(&s).unwrap();
    assert_eq!(lvm::State::from_json(&wire).unwrap(), s);
}
#[test]
fn source_binding_rejects_identity_injection_replacement_and_swaps() {
    let before = fixture();
    let mut bad = before.clone();
    bad.identity = None;
    assert!(!work::same_problem(&before, &bad));
    assert!(!work::same_problem(&bad, &before));
    let mut bad = before.clone();
    let ids = bad.identity.as_mut().unwrap();
    let a = ids.volumes["cold/archive"];
    let b = ids.volumes["live/ledger"];
    ids.volumes.insert("cold/archive".into(), b);
    ids.volumes.insert("live/ledger".into(), a);
    bad.validate().unwrap();
    assert!(!work::same_problem(&before, &bad));
    let mut bad = before.clone();
    let ids = bad.identity.as_mut().unwrap();
    ids.volumes.insert("cold/archive".into(), ids.next);
    ids.next += 1;
    bad.validate().unwrap();
    assert!(!work::same_problem(&before, &bad));
    let mut bad = before.clone();
    bad.activation.as_mut().unwrap().open.clear();
    assert!(!work::same_problem(&before, &bad));
    let mut bad = before.clone();
    bad.groups
        .get_mut("cold")
        .unwrap()
        .volumes
        .get_mut("archive")
        .unwrap()
        .extents
        .reverse();
    bad.validate().unwrap();
    assert!(!work::same_problem(&before, &bad));
}
#[test]
fn malformed_identity_imports_are_bounded_and_fail_closed() {
    let raw = serde_json::to_value(fixture()).unwrap();
    for mode in 0..11 {
        let mut bad = raw.clone();
        match mode {
            0 => bad["identity"]["next"] = json!(0),
            1 => bad["identity"]["next"] = json!(65537),
            2 => bad["identity"]["groups"]["live"] = json!(0),
            3 => bad["identity"]["groups"]["live"] = bad["identity"]["groups"]["cold"].clone(),
            4 => {
                bad["identity"]["volumes"]
                    .as_object_mut()
                    .unwrap()
                    .remove("live/ledger");
            }
            5 => bad["identity"]["volumes"]["/dev/live/ledger"] = json!(20),
            6 => bad["identity"]["groups"]["x".repeat(65)] = json!(20),
            7 => bad["identity"]["groups"] = json!({"a":1,"b":2,"c":3,"d":4,"e":5}),
            8 => bad["identity"]["unknown"] = json!(true),
            9 => bad["activation"] = Value::Null,
            _ => bad["provisioning"] = json!("Linear"),
        }
        assert!(
            lvm::State::from_json(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "mode{mode}"
        );
    }
}
#[test]
fn exhausted_creation_budget_refuses_atomically_but_rename_still_works() {
    let mut s = fixture();
    s.identity.as_mut().unwrap().next = 65536;
    s.validate().unwrap();
    assert_eq!(
        refused(&mut s, "lvcreate -l 1 -n spare staging").kind,
        ErrorKind::Limit
    );
    s.execute("lvrename staging buffer batch").unwrap();
    assert_eq!(s.identity.as_ref().unwrap().next, 65536);
}
#[test]
fn all_existing_319_source_states_keep_wire_bytes_and_no_rename_capability() {
    for scenario in dlms_learning_lab::pack::accepted319().unwrap() {
        if let dlms_learning_lab::engine::Goal::Lvm { definition } = &scenario.goal {
            for before in [&definition.initial, &definition.lesson.fixture] {
                let raw = serde_json::to_value(before).unwrap();
                assert!(raw.get("identity").is_none());
                let mut s = before.clone();
                for c in [
                    "lvrename cold archive records",
                    "vgrename cold warehouse",
                    "lvrename --force cold archive records",
                ] {
                    let e = refused(&mut s, c);
                    assert_eq!(e.kind, ErrorKind::Unsupported);
                    assert_eq!(e.message, before.command_help());
                }
                assert_eq!(serde_json::to_value(&s).unwrap(), raw);
            }
        }
    }
}
