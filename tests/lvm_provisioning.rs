//! Pure bounded allocation; no real LVM/device/reference command runs.
use dlms_learning_lab::lvm::{self, ErrorKind, MIB, provision::Profile};
use serde_json::json;
fn original() -> lvm::State {
    let d: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/lvm-growth-v1.json")).unwrap();
    lvm::State::from_json(
        &serde_json::to_vec(&d["scenarios"][0]["goal"]["definition"]["initial"]).unwrap(),
    )
    .unwrap()
}
fn fixture() -> lvm::State {
    let mut s = original();
    s.provisioning = Some(Profile::Linear);
    for p in ["/dev/newa", "/dev/newb"] {
        s.devices.insert(
            p.into(),
            lvm::Device {
                pv_bytes: None,
                bytes: 33 * MIB,
                role: lvm::DeviceRole::Blank,
                group: String::new(),
            },
        );
    }
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
fn vgcreate_initializes_blank_pvs_or_accepts_separate_pvcreate_equivalently() {
    let mut a = fixture();
    let mut b = a.clone();
    a.execute("vgcreate intake /dev/newa /dev/newb").unwrap();
    b.execute("pvcreate /dev/newb /dev/newa").unwrap();
    b.execute("vgcreate intake /dev/newb /dev/newa").unwrap();
    assert_eq!(a, b);
    assert_eq!(a.groups["intake"].extent_bytes, 4 * MIB);
    assert_eq!(a.free_extents("intake").unwrap(), 16);
    assert_eq!(a.groups["records"], original().groups["records"]);
}
#[test]
fn new_lv_uses_actual_free_mapping_and_rounds_size_without_creating_a_filesystem() {
    let mut s = fixture();
    let before = s.groups["records"].clone();
    s.execute("lvcreate -L 9M -n staging records").unwrap();
    assert_eq!(s.lv_bytes("records/staging").unwrap(), 12 * MIB);
    assert!(s.volume("records/staging").unwrap().1.filesystem.is_none());
    assert_eq!(s.free_extents("records").unwrap(), 1);
    for (n, v) in before.volumes {
        assert_eq!(s.groups["records"].volumes[&n], v);
    }
    assert!(
        s.execute("lvs")
            .unwrap()
            .contains("staging records 12582912 3 linear")
    );
}
#[test]
fn bytes_extents_name_order_and_long_flags_have_equivalent_results() {
    let mut results = Vec::new();
    for c in [
        "lvcreate -L 12M -n staging records",
        "lvcreate -n staging -l 3 records",
        "lvcreate --name staging --size 12m records",
        "lvcreate --extents 3 --name staging records",
    ] {
        let mut s = fixture();
        s.execute(c).unwrap();
        results.push(s);
    }
    assert!(results.windows(2).all(|p| p[0] == p[1]));
}
#[test]
fn explicit_group_extents_change_real_capacity_calculation() {
    let mut s = fixture();
    s.execute("vgcreate --physicalextentsize 2M intake /dev/newa")
        .unwrap();
    assert_eq!(s.free_extents("intake").unwrap(), 16);
    s.execute("lvcreate -n staging -L 5M intake").unwrap();
    assert_eq!(s.lv_bytes("intake/staging").unwrap(), 6 * MIB);
    assert_eq!(s.free_extents("intake").unwrap(), 13);
    assert_eq!(
        refused(&mut s, "vgcreate -s 3M wrong /dev/newb").kind,
        ErrorKind::Unsupported
    );
}
#[test]
fn invalid_or_occupied_multi_device_group_never_partially_initializes() {
    let mut s = fixture();
    for c in [
        "vgcreate intake /dev/newa /dev/recordsforeign",
        "vgcreate intake /dev/newa /dev/recordsa",
        "vgcreate intake /dev/newa /dev/missing",
        "vgcreate intake /dev/newa /dev/newa",
        "vgcreate records /dev/newa",
    ] {
        refused(&mut s, c);
    }
    assert_eq!(s.devices["/dev/newa"].role, lvm::DeviceRole::Blank);
}
#[test]
fn wrong_sizes_collisions_and_relative_creation_do_not_publish_candidates() {
    let mut s = fixture();
    for c in [
        "lvcreate -L 40M -n staging records",
        "lvcreate -l 0 -n staging records",
        "lvcreate -l 4097 -n staging records",
        "lvcreate -L 18446744073709551615G -n staging records",
        "lvcreate -l +2 -n staging records",
        "lvcreate -L 4M -n reports records",
        "lvcreate -L 4M -n staging absent",
        "lvcreate -L 4M -l 1 -n staging records",
        "lvcreate -n staging records",
    ] {
        refused(&mut s, c);
    }
}
#[test]
fn raw_remove_returns_extents_recreate_recovers_and_filesystem_removal_is_refused() {
    let mut s = fixture();
    s.execute("lvcreate -l 2 -n staging records").unwrap();
    let good = s.clone();
    s.execute("lvremove --yes /dev/records/staging").unwrap();
    assert_eq!(s.free_extents("records").unwrap(), 4);
    assert!(s.volume("records/staging").is_err());
    s.execute("lvcreate -n staging --size 8M records").unwrap();
    assert_eq!(s, good);
    refused(&mut s, "lvremove -y records/reports");
    refused(&mut s, "lvremove records/staging");
}
#[test]
fn all_free_uses_remaining_pool_not_total_and_exhaustion_is_honest() {
    let mut s = fixture();
    s.execute("lvcreate -l 100%FREE -n staging records")
        .unwrap();
    assert_eq!(s.lv_bytes("records/staging").unwrap(), 16 * MIB);
    assert_eq!(s.free_extents("records").unwrap(), 0);
    assert_eq!(
        refused(&mut s, "lvcreate -l 1 -n another records").kind,
        ErrorKind::NoSpace
    );
    refused(&mut s, "lvcreate -l 100%FREE -n another records");
}
#[test]
fn old_profile_is_absent_and_cannot_gain_new_commands_from_saved_work() {
    let mut old = original();
    let value = serde_json::to_value(&old).unwrap();
    assert!(value.get("provisioning").is_none());
    for c in [
        "vgcreate intake /dev/recordsa",
        "lvcreate -l 1 -n staging records",
        "lvremove -y records/neighbor",
    ] {
        assert_eq!(refused(&mut old, c).kind, ErrorKind::Unsupported);
    }
    let mut invalid = value;
    invalid["provisioning"] = json!("Thin");
    assert!(lvm::State::from_json(&serde_json::to_vec(&invalid).unwrap()).is_err());
}
#[test]
fn unsupported_modes_shell_and_path_escapes_fail_without_mutation() {
    let mut s = fixture();
    for c in [
        "lvcreate --snapshot -L 4M -n snapshot records/reports",
        "lvcreate --type raid1 -L 4M -n staging records",
        "lvcreate -L4M -n staging records",
        "lvcreate -L 4M -n ../outside records",
        "vgcreate intake /dev/../outside",
        "lvremove -fy records/reports",
        "vgremove intake",
        "lvcreate -L 4M -n staging records; id",
        "lvcreate -L $(id) -n staging records",
    ] {
        refused(&mut s, c);
    }
    refused(&mut s, &"x".repeat(2049));
}
#[test]
fn json_roundtrip_retains_real_allocations_and_overlapping_imports_fail() {
    let mut s = fixture();
    s.execute("vgcreate intake /dev/newa").unwrap();
    s.execute("lvcreate -L 8M -n staging intake").unwrap();
    assert_eq!(
        lvm::State::from_json(&serde_json::to_vec(&s).unwrap()).unwrap(),
        s
    );
    let mut bad = serde_json::to_value(&s).unwrap();
    bad["groups"]["intake"]["volumes"]["staging"]["extents"][1] =
        bad["groups"]["intake"]["volumes"]["staging"]["extents"][0].clone();
    assert!(lvm::State::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
}

#[test]
fn unsupported_creation_profile_commands_disclose_the_current_subset_without_old_contradictions() {
    let mut enabled = fixture();
    let e = refused(&mut enabled, "mkfs.ext4 /dev/records/staging");
    assert_eq!(e.kind, ErrorKind::Unsupported);
    assert_eq!(e.message, lvm::provision::HELP);
    let mut old = original();
    let e = refused(&mut old, "mkfs.ext4 /dev/records/reports");
    assert_eq!(e.message, lvm::HELP);
}

#[test]
fn creation_respects_internal_names_and_existing_group_volume_limits() {
    let mut s = fixture();
    for name in [
        "snapshot",
        "pvmove",
        "bad_cdata",
        "bad_tmeta",
        "bad_vorigin",
    ] {
        refused(&mut s, &format!("lvcreate -L 4M -n {name} records"));
    }
    refused(&mut s, "vgcreate recordsa /dev/newa");
    s.devices.get_mut("/dev/newa").unwrap().bytes = 257 * MIB;
    s.execute("vgcreate intake /dev/newa").unwrap();
    for n in 0..14 {
        s.execute(&format!("lvcreate -l 1 -n item{n} intake"))
            .unwrap();
    }
    assert_eq!(
        refused(&mut s, "lvcreate -l 1 -n extra intake").kind,
        ErrorKind::Limit
    );
    // A new group consumes an existing supplied device, never creates one.
    let mut s = fixture();
    for n in 0..3 {
        let device = format!("/dev/extra{n}");
        s.devices.insert(
            device.clone(),
            lvm::Device {
                pv_bytes: None,
                bytes: 17 * MIB,
                role: lvm::DeviceRole::Blank,
                group: String::new(),
            },
        );
        s.execute(&format!("vgcreate pool{n} {device}")).unwrap();
    }
    assert_eq!(
        refused(&mut s, "vgcreate toomany /dev/newb").kind,
        ErrorKind::Limit
    );
}
