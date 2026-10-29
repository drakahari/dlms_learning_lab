//! Pure virtual LVM capability, not an authored/counted lab or host LVM test.
use dlms_learning_lab::lvm::{
    self, BLOCK_BYTES, DATA_START, Device, DeviceRole, ErrorKind, Extent, File, Filesystem,
    LogicalVolume, MIB, State, VolumeGroup,
};
use std::collections::BTreeMap;
fn fixture() -> State {
    State {
        version: 1,
        devices: BTreeMap::from([
            (
                "/dev/vda".into(),
                Device {
                    bytes: 17 * MIB,
                    role: DeviceRole::PhysicalVolume,
                    group: "data".into(),
                },
            ),
            (
                "/dev/vdb".into(),
                Device {
                    bytes: 33 * MIB,
                    role: DeviceRole::Blank,
                    group: String::new(),
                },
            ),
            (
                "/dev/vdc".into(),
                Device {
                    bytes: 33 * MIB,
                    role: DeviceRole::Foreign,
                    group: String::new(),
                },
            ),
            (
                "/dev/vdd".into(),
                Device {
                    bytes: 17 * MIB,
                    role: DeviceRole::PhysicalVolume,
                    group: "peer".into(),
                },
            ),
        ]),
        groups: BTreeMap::from([
            (
                "data".into(),
                VolumeGroup {
                    extent_bytes: 4 * MIB,
                    volumes: BTreeMap::from([
                        (
                            "reports".into(),
                            LogicalVolume {
                                extents: vec![
                                    Extent {
                                        pv: "/dev/vda".into(),
                                        index: 0,
                                    },
                                    Extent {
                                        pv: "/dev/vda".into(),
                                        index: 1,
                                    },
                                ],
                                filesystem: Some(Filesystem {
                                    blocks: 8 * MIB / BLOCK_BYTES,
                                    files: files(),
                                }),
                            },
                        ),
                        (
                            "neighbor".into(),
                            LogicalVolume {
                                extents: vec![Extent {
                                    pv: "/dev/vda".into(),
                                    index: 3,
                                }],
                                filesystem: None,
                            },
                        ),
                    ]),
                },
            ),
            (
                "peer".into(),
                VolumeGroup {
                    extent_bytes: 4 * MIB,
                    volumes: BTreeMap::from([(
                        "archive".into(),
                        LogicalVolume {
                            extents: vec![Extent {
                                pv: "/dev/vdd".into(),
                                index: 0,
                            }],
                            filesystem: Some(Filesystem {
                                blocks: 4 * MIB / BLOCK_BYTES,
                                files: BTreeMap::from([(
                                    "/keep".into(),
                                    File {
                                        contents: "peer unchanged\n".into(),
                                    },
                                )]),
                            }),
                        },
                    )]),
                },
            ),
        ]),
    }
}
fn files() -> BTreeMap<String, File> {
    BTreeMap::from([
        (
            "/report.txt".into(),
            File {
                contents: "original report\n".into(),
            },
        ),
        (
            "/notes".into(),
            File {
                contents: "Mira's preserved notes\n".into(),
            },
        ),
    ])
}
fn run(s: &mut State, command: &str) {
    s.execute(command)
        .unwrap_or_else(|e| panic!("{command}: {e}"));
    s.validate().unwrap();
}
fn rows(s: &State) -> Vec<bool> {
    lvm::growth_checks(s, "data/reports", 24 * MIB, 24 * MIB, &files())
        .iter()
        .map(|r| r.0)
        .collect()
}
fn refusal(s: &mut State, c: &str) {
    let before = s.clone();
    assert!(s.execute(c).is_err(), "unexpected acceptance: {c}");
    assert_eq!(*s, before, "refusal must retain entire state: {c}");
}
#[test]
fn reference_grows_backing_then_filesystem_and_preserves_mapping_and_data() {
    let mut s = fixture();
    s.validate().unwrap();
    assert_eq!(rows(&s), vec![false, false, true]);
    let before = s.clone();
    run(&mut s, "pvcreate /dev/vdb");
    assert_eq!(s.devices["/dev/vdb"].role, DeviceRole::PhysicalVolume);
    assert!(s.devices["/dev/vdb"].group.is_empty());
    run(&mut s, "vgextend data /dev/vdb");
    assert_eq!(s.free_extents("data").unwrap(), 9);
    run(&mut s, "lvextend -L +16M /dev/data/reports");
    assert_eq!(s.lv_bytes("data/reports").unwrap(), 24 * MIB);
    assert_eq!(rows(&s), vec![true, false, true]); // LV success is not filesystem success.
    assert_eq!(s.filesystem_bytes("data/reports").unwrap(), 8 * MIB);
    assert_eq!(
        &s.groups["data"].volumes["reports"].extents[..2],
        before.groups["data"].volumes["reports"].extents.as_slice()
    );
    run(&mut s, "resize2fs data/reports");
    assert!(rows(&s).iter().all(|b| *b));
    assert_eq!(
        s.read_file("data/reports", "/report.txt").unwrap(),
        "original report\n"
    );
    assert_eq!(s.groups["peer"], before.groups["peer"]);
    assert_eq!(
        s.groups["data"].volumes["neighbor"],
        before.groups["data"].volumes["neighbor"]
    );
    assert_eq!(s.devices["/dev/vdc"], before.devices["/dev/vdc"]);
}
#[test]
fn legitimate_absolute_relative_extent_size_and_automatic_pv_initialization_alternatives() {
    for command in [
        "lvextend -l 6 data/reports",
        "lvextend --extents +4 data/reports",
        "lvextend -L 24m data/reports",
        "lvextend --size +16 data/reports",
    ] {
        let mut s = fixture();
        run(&mut s, "vgextend data /dev/vdb");
        run(&mut s, command);
        run(&mut s, "resize2fs /dev/data/reports 6144");
        assert_eq!(rows(&s), vec![true, true, true]);
    }
    let mut s = fixture();
    run(&mut s, "vgextend data /dev/vdb");
    run(&mut s, "lvextend -l +100%FREE data/reports");
    assert_eq!(s.free_extents("data").unwrap(), 0);
    assert_eq!(s.lv_bytes("data/reports").unwrap(), 44 * MIB);
    run(&mut s, "resize2fs data/reports 24M");
    assert_eq!(rows(&s), vec![true, true, true]);
    assert_eq!(s.filesystem_bytes("data/reports").unwrap(), 24 * MIB);
}
#[test]
fn first_fit_allocation_uses_real_free_holes_and_extends_without_reassigning_prefix() {
    let mut s = fixture();
    run(&mut s, "lvextend -l +1 data/reports");
    assert_eq!(
        s.groups["data"].volumes["reports"].extents.last().unwrap(),
        &Extent {
            pv: "/dev/vda".into(),
            index: 2
        }
    );
    assert_eq!(s.free_extents("data").unwrap(), 0);
    run(&mut s, "vgextend data /dev/vdb");
    run(&mut s, "lvextend -l +1 data/reports");
    assert_eq!(
        s.groups["data"].volumes["reports"].extents.last().unwrap(),
        &Extent {
            pv: "/dev/vdb".into(),
            index: 0
        }
    );
    assert_eq!(s.groups["data"].volumes["neighbor"].extents[0].index, 3);
}
#[test]
fn extent_rounding_uses_binary_units_and_drops_only_unusable_pv_tail() {
    let mut s = fixture();
    s.devices.get_mut("/dev/vdb").unwrap().bytes = 34 * MIB + 512;
    run(&mut s, "vgextend data /dev/vdb");
    assert_eq!(s.free_extents("data").unwrap(), 9);
    run(&mut s, "lvextend -L +1M data/reports");
    assert_eq!(s.lv_bytes("data/reports").unwrap(), 12 * MIB);
    assert_eq!(s.filesystem_bytes("data/reports").unwrap(), 8 * MIB);
    run(&mut s, "lvextend -L 13M data/reports");
    assert_eq!(s.lv_bytes("data/reports").unwrap(), 16 * MIB);
    run(&mut s, "resize2fs data/reports 12m");
    assert_eq!(s.filesystem_bytes("data/reports").unwrap(), 12 * MIB);
    let before = s.clone();
    run(&mut s, "resize2fs data/reports 12M");
    assert_eq!(s, before); // Real no-op at same size.
}
#[test]
fn growth_before_backing_capacity_wrong_volume_and_overallocation_fail_without_effects() {
    let mut s = fixture();
    assert_eq!(
        s.execute("resize2fs data/reports 24M").unwrap_err().kind,
        ErrorKind::NoSpace
    );
    for cmd in [
        "lvextend -l +2 data/reports",
        "lvextend -l +1 peer/reports",
        "lvextend -L 64M data/reports",
        "vgextend no-such /dev/vdb",
        "vgextend data /dev/vdd",
        "resize2fs data/neighbor",
    ] {
        refusal(&mut s, cmd);
    }
    assert_eq!(rows(&s), vec![false, false, true]);
}
#[test]
fn foreign_device_duplicates_and_multi_device_late_failure_do_not_partly_initialize() {
    let mut s = fixture();
    for cmd in [
        "pvcreate /dev/vdc",
        "pvcreate /dev/vda",
        "pvcreate /dev/vdb /dev/vdc",
        "vgextend data /dev/vdb /dev/vdc",
        "pvcreate /dev/vdb /dev/vdb",
        "vgextend data /dev/vdb /dev/vdb",
        "pvcreate /dev/not-in-inventory",
    ] {
        refusal(&mut s, cmd);
    }
    assert_eq!(s.devices["/dev/vdb"].role, DeviceRole::Blank);
}
#[test]
fn unsupported_shrink_force_shell_path_alias_and_unknown_syntax_are_honest() {
    let mut s = fixture();
    for cmd in [
        "lvreduce -L 4M data/reports",
        "lvextend -L 4M data/reports",
        "resize2fs data/reports 1M",
        "pvcreate -ff /dev/vdc",
        "lvextend -r -L +4M data/reports",
        "lvextend -L 8.5M data/reports",
        "lvextend -L +1T data/reports",
        "lvextend -l 100%FREE data/reports",
        "lvextend -l +100%VG data/reports",
        "pvs --reportformat json",
        "vgcreate new /dev/vdb",
        "lvcreate -L 4M -n new data",
        "mount /dev/data/reports /data",
        "pvs; echo escape",
        "pvs\n",
        "pvcreate /dev/../etc/passwd",
        "resize2fs /dev/mapper/data-reports",
        "resize2fs data/reports 1é",
    ] {
        refusal(&mut s, cmd);
    }
    let e = s.execute("resize2fs data/reports 1M").unwrap_err();
    assert!(e.message.contains("real offline resize2fs can shrink"));
    assert!(lvm::HELP.contains("Real LVM"));
}
#[test]
fn overflow_and_raw_command_bounds_never_panic_or_change_state() {
    let mut s = fixture();
    for cmd in [
        "lvextend -l +18446744073709551615 data/reports",
        "lvextend -L 18446744073709551615G data/reports",
        "lvextend -l 18446744073709551616 data/reports",
        "resize2fs data/reports 18446744073709551615",
        "lvextend -l 4097 data/reports",
    ] {
        refusal(&mut s, cmd);
    }
    refusal(&mut s, &"x".repeat(2049));
    refusal(&mut s, "resize2fs data/reports ☃");
}
#[test]
fn reports_are_read_only_and_derived_from_current_extent_ownership() {
    let mut s = fixture();
    let before = s.clone();
    let p = s.execute("pvs").unwrap();
    let v = s.execute("vgs").unwrap();
    let l = s.execute("lvs").unwrap();
    assert_eq!(s, before);
    assert!(!p.contains("/dev/vdb") && !p.contains("/dev/vdc"));
    assert!(p.contains("/dev/vda data lvm2 16777216 4194304"));
    assert!(v.contains("data 1 2 16777216 4194304 4194304"));
    assert!(l.contains("reports data 8388608 2 linear"));
    run(&mut s, "vgextend data /dev/vdb");
    run(&mut s, "lvextend -l +1 data/reports");
    let p = s.execute("pvs").unwrap();
    assert!(p.contains("/dev/vda data lvm2 16777216 0"));
    assert!(p.contains("/dev/vdb data lvm2 33554432 33554432"));
    assert!(
        s.execute("lvs")
            .unwrap()
            .contains("reports data 12582912 3 linear")
    );
}
#[test]
fn multiple_vg_extent_sizes_and_small_pv_alignment_are_validated() {
    let mut s = fixture();
    s.groups.get_mut("peer").unwrap().extent_bytes = 2 * MIB;
    s.groups
        .get_mut("peer")
        .unwrap()
        .volumes
        .get_mut("archive")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .blocks = 2 * MIB / BLOCK_BYTES;
    s.validate().unwrap();
    assert_eq!(s.lv_bytes("peer/archive").unwrap(), 2 * MIB);
    assert_eq!(s.free_extents("peer").unwrap(), 7);
    run(&mut s, "lvextend -L +1M peer/archive");
    assert_eq!(s.lv_bytes("peer/archive").unwrap(), 4 * MIB);
    assert_eq!(s.filesystem_bytes("peer/archive").unwrap(), 2 * MIB);
    run(&mut s, "resize2fs peer/archive");
    assert_eq!(s.filesystem_bytes("peer/archive").unwrap(), 4 * MIB);
    s.devices.get_mut("/dev/vdb").unwrap().bytes = DATA_START + 512;
    s.validate().unwrap();
    refusal(&mut s, "vgextend data /dev/vdb");
}
#[test]
fn saved_state_roundtrip_keeps_exact_files_and_new_requirements_cannot_inherit_success() {
    let mut s = fixture();
    run(&mut s, "vgextend data /dev/vdb");
    run(&mut s, "lvextend -L 24M data/reports");
    run(&mut s, "resize2fs data/reports");
    assert_eq!(rows(&s), vec![true, true, true]);
    let raw = serde_json::to_vec(&s).unwrap();
    let mut restored = State::from_json(&raw).unwrap();
    assert_eq!(restored, s);
    restored
        .groups
        .get_mut("data")
        .unwrap()
        .volumes
        .get_mut("reports")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .blocks = 8 * MIB / BLOCK_BYTES;
    assert_eq!(rows(&restored), vec![true, false, true]);
    restored
        .groups
        .get_mut("data")
        .unwrap()
        .volumes
        .get_mut("reports")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .get_mut("/notes")
        .unwrap()
        .contents = "damaged".into();
    assert_eq!(rows(&restored), vec![true, false, false]);
    assert_eq!(
        s.read_file("data/reports", "/notes").unwrap(),
        "Mira's preserved notes\n"
    );
}
#[test]
fn invalid_snapshots_reject_overlap_foreign_extents_missing_groups_and_filesystem_overrun() {
    let base = fixture();
    for n in 0..9 {
        let mut s = base.clone();
        match n {
            0 => {
                s.groups
                    .get_mut("data")
                    .unwrap()
                    .volumes
                    .get_mut("reports")
                    .unwrap()
                    .extents[1]
                    .index = 0
            }
            1 => {
                s.groups
                    .get_mut("data")
                    .unwrap()
                    .volumes
                    .get_mut("reports")
                    .unwrap()
                    .extents[1]
                    .pv = "/dev/vdd".into()
            }
            2 => {
                s.groups
                    .get_mut("data")
                    .unwrap()
                    .volumes
                    .get_mut("reports")
                    .unwrap()
                    .extents[1]
                    .index = 4
            }
            3 => s.devices.get_mut("/dev/vda").unwrap().group = "absent".into(),
            4 => s.groups.get_mut("data").unwrap().extent_bytes = 3 * MIB,
            5 => {
                s.groups
                    .get_mut("data")
                    .unwrap()
                    .volumes
                    .get_mut("reports")
                    .unwrap()
                    .filesystem
                    .as_mut()
                    .unwrap()
                    .blocks = 9 * MIB / BLOCK_BYTES
            }
            6 => s.devices.get_mut("/dev/vda").unwrap().bytes = DATA_START,
            7 => {
                s.groups
                    .get_mut("data")
                    .unwrap()
                    .volumes
                    .get_mut("reports")
                    .unwrap()
                    .filesystem
                    .as_mut()
                    .unwrap()
                    .files
                    .insert(
                        "/../secret".into(),
                        File {
                            contents: "bad".into(),
                        },
                    );
            }
            _ => {
                s.version = 2;
            }
        };
        assert!(s.validate().is_err(), "mutation {n}");
        let raw = serde_json::to_vec(&s).unwrap();
        assert!(State::from_json(&raw).is_err());
        let before = s.clone();
        assert!(s.execute("lvs").is_err());
        assert_eq!(s, before);
    }
}
#[test]
fn raw_codec_rejects_duplicates_unknown_fields_excess_collections_and_data() {
    let s = fixture();
    let raw = serde_json::to_string(&s).unwrap();
    assert_eq!(State::from_json(raw.as_bytes()).unwrap(), s);
    assert!(
        State::from_json(
            raw.replacen("\"version\":1", "\"version\":1,\"version\":1", 1)
                .as_bytes()
        )
        .is_err()
    );
    assert!(
        State::from_json(
            raw.replacen("\"version\":1", "\"version\":1,\"unknown\":0", 1)
                .as_bytes()
        )
        .is_err()
    );
    assert!(State::from_json(&vec![b' '; lvm::MAX_STATE_BYTES + 1]).is_err());
    let mut value = serde_json::to_value(&s).unwrap();
    value["devices"]["/dev/alias"] = value["devices"]["/dev/vda"].clone();
    value["devices"]["/dev/vda"]["group"] = "".into();
    assert!(State::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut s = fixture();
    s.groups
        .get_mut("data")
        .unwrap()
        .volumes
        .get_mut("reports")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .get_mut("/notes")
        .unwrap()
        .contents = "x".repeat(lvm::MAX_FILE_BYTES + 1);
    assert!(State::from_json(&serde_json::to_vec(&s).unwrap()).is_err());
}
#[test]
fn streaming_bounds_refuse_poison_excess_before_its_body() {
    let d = serde_json::to_string(&Device {
        bytes: 17 * MIB,
        role: DeviceRole::Blank,
        group: String::new(),
    })
    .unwrap();
    let entries = (0..8)
        .map(|n| format!("\"/dev/test{n}\":{d}"))
        .collect::<Vec<_>>()
        .join(",");
    let raw = format!(
        "{{\"version\":1,\"devices\":{{{entries},\"/dev/excess\":POISON_MUST_NOT_BE_READ}},\"groups\":{{}}}}"
    );
    let e = State::from_json(raw.as_bytes()).unwrap_err();
    assert!(
        e.message.contains("map count/duplicate before value"),
        "{e}"
    );
    let raw = format!(
        "{{\"version\":1,\"devices\":{{\"/dev/test\":{d},\"/dev/test\":POISON_MUST_NOT_BE_READ}},\"groups\":{{}}}}"
    );
    let e = State::from_json(raw.as_bytes()).unwrap_err();
    assert!(
        e.message.contains("map count/duplicate before value"),
        "{e}"
    );
    let e = "{\"pv\":\"/dev/vda\",\"index\":0}";
    let entries = std::iter::repeat_n(e, 4096).collect::<Vec<_>>().join(",");
    let raw = format!("{{\"extents\":[{entries},POISON_MUST_NOT_BE_READ],\"filesystem\":null}}");
    let e = serde_json::from_str::<LogicalVolume>(&raw).unwrap_err();
    assert!(e.to_string().contains("sequence bound before excess body"));
}

#[test]
fn gibibyte_input_and_maximum_extent_boundary_are_math_not_command_matches() {
    for (cmd, expected) in [
        ("lvextend -L 1G data/reports", 1024 * MIB),
        ("lvextend -L +1g data/reports", 1032 * MIB),
    ] {
        let mut s = fixture();
        s.devices.get_mut("/dev/vdb").unwrap().bytes = 2048 * MIB + DATA_START;
        run(&mut s, "vgextend data /dev/vdb");
        run(&mut s, cmd);
        assert_eq!(s.lv_bytes("data/reports").unwrap(), expected);
        assert_eq!(s.filesystem_bytes("data/reports").unwrap(), 8 * MIB);
        run(&mut s, "resize2fs data/reports");
        assert_eq!(s.filesystem_bytes("data/reports").unwrap(), expected);
        assert_eq!(
            s.read_file("data/reports", "/report.txt").unwrap(),
            "original report\n"
        );
    }
    let mut s = fixture();
    s.devices.get_mut("/dev/vda").unwrap().bytes = 4096 * 4 * MIB + DATA_START;
    s.groups.get_mut("data").unwrap().volumes.remove("neighbor");
    s.validate().unwrap();
    run(&mut s, "lvextend -l 4096 data/reports");
    assert_eq!(s.groups["data"].volumes["reports"].extents.len(), 4096);
    assert_eq!(s.free_extents("data").unwrap(), 0);
    refusal(&mut s, "lvextend -l +1 data/reports");
    let raw = serde_json::to_vec(&s).unwrap();
    assert_eq!(State::from_json(&raw).unwrap(), s);
}
