//! Pure recognized-PV capacity, not authored curriculum or real-device acceptance.
use dlms_learning_lab::{
    lvm::{self, DeviceRole, ErrorKind, MIB, pv_capacity},
    lvm_workspace as work,
};
use serde_json::{Value, json};
fn old() -> work::Definition {
    let p: Value = serde_json::from_str(include_str!("../scenarios/lvm-growth-v1.json")).unwrap();
    serde_json::from_value(p["scenarios"][0]["goal"]["definition"].clone()).unwrap()
}
fn enable(s: &mut lvm::State) {
    s.pv_capacity = Some(pv_capacity::Profile::MetadataSize);
    for d in s.devices.values_mut() {
        if d.role == DeviceRole::PhysicalVolume {
            d.pv_bytes = Some(d.bytes);
        }
    }
}
fn fixture() -> lvm::State {
    let p: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-activation-lessons-v1.json")).unwrap();
    let mut s: lvm::State =
        serde_json::from_value(p["scenarios"][0]["goal"]["definition"]["initial"].clone()).unwrap();
    s.provisioning = None;
    s.activation = None;
    enable(&mut s);
    s.devices.get_mut("/dev/reporta").unwrap().bytes = 65 * MIB;
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
fn device_pv_lv_and_filesystem_are_separate_real_state_layers() {
    let mut s = fixture();
    let before = s.clone();
    let file = s
        .read_file("reports/ledger", "/keep.txt")
        .unwrap()
        .to_owned();
    assert_eq!(s.free_extents("reports").unwrap(), 6);
    assert_eq!(s.devices["/dev/reporta"].pv_bytes, Some(33 * MIB));
    s.execute("pvresize /dev/reporta").unwrap();
    assert_eq!(s.devices["/dev/reporta"].bytes, 65 * MIB);
    assert_eq!(s.devices["/dev/reporta"].pv_bytes, Some(65 * MIB));
    assert_eq!(s.free_extents("reports").unwrap(), 14);
    assert_eq!(s.groups, before.groups);
    assert_eq!(s.lv_bytes("reports/ledger").unwrap(), 8 * MIB);
    assert_eq!(s.filesystem_bytes("reports/ledger").unwrap(), 8 * MIB);
    s.execute("lvextend -L 24M reports/ledger").unwrap();
    assert_eq!(s.filesystem_bytes("reports/ledger").unwrap(), 8 * MIB);
    s.execute("resize2fs /dev/reports/ledger").unwrap();
    assert_eq!(s.filesystem_bytes("reports/ledger").unwrap(), 24 * MIB);
    assert_eq!(s.read_file("reports/ledger", "/keep.txt").unwrap(), file);
    assert_eq!(s.devices["/dev/reservea"], before.devices["/dev/reservea"]);
}
#[test]
fn explicit_size_and_autodetection_accept_equivalent_supported_inputs() {
    let mut results = Vec::new();
    for c in [
        "pvresize /dev/reporta",
        "pvresize --setphysicalvolumesize 65M /dev/reporta",
        "pvresize --setphysicalvolumesize 65m /dev/reporta",
        "pvresize --setphysicalvolumesize 65 /dev/reporta",
    ] {
        let mut s = fixture();
        s.execute(c).unwrap();
        results.push(s);
    }
    assert!(results.windows(2).all(|s| s[0] == s[1]));
    let mut s = fixture();
    s.execute("pvresize --setphysicalvolumesize 1G /dev/reporta")
        .unwrap_err();
}
#[test]
fn high_physical_placement_prevents_shrink_despite_large_total_free_space() {
    let mut s = fixture();
    s.groups
        .get_mut("reports")
        .unwrap()
        .volumes
        .get_mut("ledger")
        .unwrap()
        .extents[1]
        .index = 4;
    s.validate().unwrap();
    assert_eq!(s.free_extents("reports").unwrap(), 6);
    assert_eq!(
        pv_capacity::highest_allocated(&s, "/dev/reporta").unwrap(),
        Some(4)
    );
    let e = refused(&mut s, "pvresize --setphysicalvolumesize 17M /dev/reporta");
    assert_eq!(e.kind, ErrorKind::Occupied);
    assert!(e.message.contains("extent 4"));
    assert!(e.message.contains("total free space"));
    let before = s.groups.clone();
    s.execute("pvresize --setphysicalvolumesize 25M /dev/reporta")
        .unwrap();
    assert_eq!(s.free_extents("reports").unwrap(), 4);
    assert_eq!(s.groups, before);
    assert_eq!(s.devices["/dev/reporta"].bytes, 65 * MIB);
    s.execute("pvresize --setphysicalvolumesize 21M /dev/reporta")
        .unwrap();
    assert_eq!(s.free_extents("reports").unwrap(), 3); // end just beyond highest allocated index4
    assert!(
        refused(&mut s, "pvresize --setphysicalvolumesize 20M /dev/reporta")
            .message
            .contains("extent 4")
    );
}
#[test]
fn multi_device_checks_publish_no_partial_update_on_missing_or_occupied_end() {
    let mut s = fixture();
    s.groups
        .get_mut("reserve")
        .unwrap()
        .volumes
        .get_mut("spare")
        .unwrap()
        .extents[0]
        .index = 3;
    s.validate().unwrap();
    assert_eq!(
        refused(
            &mut s,
            "pvresize --setphysicalvolumesize 9M /dev/reporta /dev/reservea"
        )
        .kind,
        ErrorKind::Occupied
    );
    assert_eq!(
        refused(&mut s, "pvresize /dev/reporta /dev/absent").kind,
        ErrorKind::Unknown
    );
    s.execute("pvresize /dev/reporta /dev/reservea").unwrap();
    s.validate().unwrap();
    assert_eq!(s.devices["/dev/reporta"].pv_bytes, Some(65 * MIB));
}
#[test]
fn current_reports_and_roundtrip_follow_metadata_not_physical_geometry() {
    let mut s = fixture();
    let initial = s.execute("pvs").unwrap();
    assert!(initial.contains("/dev/reporta reports lvm2 33554432 25165824"));
    s.execute("pvresize /dev/reporta").unwrap();
    let current = s.execute("pvs").unwrap();
    assert!(current.contains("/dev/reporta reports lvm2 67108864 58720256"));
    assert_eq!(
        lvm::State::from_json(&serde_json::to_vec(&s).unwrap()).unwrap(),
        s
    );
    assert_eq!(
        pv_capacity::highest_allocated(&s, "/dev/absent")
            .unwrap_err()
            .kind,
        ErrorKind::Unknown
    );
}
#[test]
fn unsupported_syntax_escape_size_bounds_and_creation_profiles_fail_honestly() {
    let mut s = fixture();
    for c in [
        "pvresize --force /dev/reporta",
        "pvresize -y /dev/reporta",
        "pvresize --setphysicalvolumesize +33M /dev/reporta",
        "pvresize --setphysicalvolumesize 1.5G /dev/reporta",
        "pvresize --setphysicalvolumesize 33K /dev/reporta",
        "pvresize --setphysicalvolumesize 0 /dev/reporta",
        "pvresize --setphysicalvolumesize 1M /dev/reporta",
        "pvresize --setphysicalvolumesize 66M /dev/reporta",
        "pvresize /dev/reporta /dev/reporta",
        "pvresize /dev/../reorta",
        "pvresize /dev/reporta; id",
        "pvresize $(id)",
        "vgcreate new /dev/reporta",
        "vgchange -ay reports",
        "lvrename reports ledger report",
    ] {
        refused(&mut s, c);
    }
    assert_eq!(refused(&mut s, &"x".repeat(2049)).kind, ErrorKind::Limit);
    s.devices.get_mut("/dev/reporta").unwrap().bytes = 1024 * 1024 * MIB;
    s.validate().unwrap();
    assert_eq!(
        refused(&mut s, "pvresize /dev/reporta").kind,
        ErrorKind::Limit
    );
}
#[test]
fn malformed_metadata_and_old_source_capability_injection_are_rejected() {
    let raw = serde_json::to_value(fixture()).unwrap();
    for mode in 0..8 {
        let mut bad = raw.clone();
        match mode {
            0 => bad["pv_capacity"] = Value::Null,
            1 => bad["devices"]["/dev/reporta"]["pv_bytes"] = Value::Null,
            2 => bad["devices"]["/dev/reporta"]["pv_bytes"] = json!(66 * MIB),
            3 => bad["devices"]["/dev/reporta"]["pv_bytes"] = json!(33 * MIB + 1),
            4 => bad["devices"]["/dev/reporta"]["pv_bytes"] = json!(MIB),
            5 => bad["devices"]["/dev/reporta"]["role"] = json!("Foreign"),
            6 => bad["pv_capacity"] = json!("Unknown"),
            _ => bad["provisioning"] = json!("Lifecycle"),
        }
        assert!(
            lvm::State::from_json(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "mode{mode}"
        );
    }
    let d = old();
    let mut current = d.initial.clone();
    enable(&mut current);
    current.validate().unwrap();
    assert!(!work::same_problem(&d.initial, &current));
    let mut session = work::Session::new(&d).unwrap();
    session.current = current;
    assert!(d.bound(&session).is_err());
    let mut unadmitted = d;
    enable(&mut unadmitted.initial);
    assert!(unadmitted.shape().is_err());
}
#[test]
fn all321_existing_models_keep_absent_fields_and_original_behavior() {
    let all = dlms_learning_lab::pack::accepted321().unwrap();
    assert_eq!(all.len(), 321);
    for lab in all {
        if let dlms_learning_lab::engine::Goal::Lvm { definition } = &lab.goal {
            for before in [&definition.initial, &definition.lesson.fixture] {
                let raw = serde_json::to_value(before).unwrap();
                assert!(raw.get("pv_capacity").is_none());
                assert!(
                    raw["devices"]
                        .as_object()
                        .unwrap()
                        .values()
                        .all(|d| d.get("pv_bytes").is_none())
                );
                let mut s = before.clone();
                let e = refused(&mut s, "pvresize /dev/example");
                assert_eq!(e.kind, ErrorKind::Unsupported);
                assert_eq!(e.message, before.command_help());
                assert_eq!(serde_json::to_value(s).unwrap(), raw);
            }
        }
    }
}

#[test]
fn initialization_and_joining_keep_recognized_capacity_consistent_without_auto_refresh() {
    let mut s = fixture();
    for path in ["/dev/newone", "/dev/newtwo"] {
        s.devices.insert(
            path.into(),
            lvm::Device {
                bytes: 17 * MIB,
                role: DeviceRole::Blank,
                pv_bytes: None,
                group: String::new(),
            },
        );
    }
    s.validate().unwrap();
    s.execute("pvcreate /dev/newone").unwrap();
    assert_eq!(s.devices["/dev/newone"].pv_bytes, Some(17 * MIB));
    s.execute("pvresize --setphysicalvolumesize 9M /dev/newone")
        .unwrap();
    let free = s.free_extents("reports").unwrap();
    s.execute("vgextend reports /dev/newone").unwrap();
    assert_eq!(s.devices["/dev/newone"].pv_bytes, Some(9 * MIB));
    assert_eq!(s.free_extents("reports").unwrap(), free + 2);
    s.execute("vgextend reports /dev/newtwo").unwrap();
    assert_eq!(s.devices["/dev/newtwo"].pv_bytes, Some(17 * MIB));
    assert_eq!(s.free_extents("reports").unwrap(), free + 6);
    s.validate().unwrap();
}
