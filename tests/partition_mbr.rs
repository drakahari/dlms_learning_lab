//! Pure opt-in capability proof, not authored lessons or host-disk conformance.
use disk::{ErrorKind, mbr};
use dlms_learning_lab::{pack, partition_fs as disk, partition_fs_workspace as work};
use serde_json::{Value, json};
fn definition() -> work::Definition {
    let p: Value =
        serde_json::from_str(include_str!("../scenarios/disk-preparation-v1.json")).unwrap();
    serde_json::from_value(p["scenarios"][0]["goal"]["definition"].clone()).unwrap()
}
fn world() -> disk::State {
    let mut s = definition().initial;
    s.mbr_profile = Some(mbr::Profile::Dos512);
    // Synthetic test world: put the retained GPT peer at vdb and blank work disk at vda.
    let protected = s.disks.remove("/dev/vda").unwrap();
    let blank = s.disks.remove("/dev/vdb").unwrap();
    s.disks.insert("/dev/vda".into(), blank);
    s.disks.insert("/dev/vdb".into(), protected);
    s.validate().unwrap();
    s
}
fn empty() -> disk::State {
    let mut s = world();
    s.execute("parted -s /dev/vda mklabel msdos").unwrap();
    s
}
fn add(s: &mut disk::State, kind: &str, start: u64, end: u64) {
    let hint = if kind == "extended" { "" } else { "ext4 " };
    s.execute(&format!(
        "parted -s /dev/vda unit s mkpart {kind} {hint}{start}s {end}s"
    ))
    .unwrap();
}
fn refuse(s: &mut disk::State, command: &str, kind: Option<ErrorKind>) {
    let before = s.to_json().unwrap();
    let e = s.execute(command).expect_err(command);
    if let Some(kind) = kind {
        assert_eq!(e.kind, kind, "{command}: {e}");
    }
    assert_eq!(
        s.to_json().unwrap(),
        before,
        "Refused operation changed state: {command}"
    );
}
#[test]
fn legacy_wire_catalog_and_gpt_goal_admission_remain_unchanged() {
    let old = definition();
    old.validate().unwrap();
    let original: Value =
        serde_json::from_str(include_str!("../scenarios/disk-preparation-v1.json")).unwrap();
    assert_eq!(
        serde_json::to_value(&old.initial).unwrap(),
        original["scenarios"][0]["goal"]["definition"]["initial"]
    );
    let old_wire = old.initial.to_json().unwrap();
    assert!(!old_wire.contains("mbr"));
    assert_eq!(disk::State::from_json(&old_wire).unwrap(), old.initial);
    let mut s = old.initial.clone();
    refuse(
        &mut s,
        "parted -s /dev/vda mklabel msdos",
        Some(ErrorKind::Unsupported),
    );
    let mut altered = old.clone();
    altered.initial.mbr_profile = Some(mbr::Profile::Dos512);
    assert!(
        altered.validate().is_err(),
        "A new model is not authority for an old GPT goal"
    );
    let catalog = pack::accepted323().unwrap();
    assert_eq!(catalog.len(), 323);
    for scenario in catalog {
        if let dlms_learning_lab::engine::Goal::PartitionFs { definition } = scenario.goal {
            assert!(definition.initial.mbr_profile.is_none());
            assert!(definition.initial.disks.values().all(|d| d.mbr.is_none()));
            definition.validate().unwrap();
        }
    }
}
#[test]
fn primary_slot_limit_is_independent_of_free_sectors_and_format_is_separate() {
    let mut s = empty();
    let protected = s.disks["/dev/vdb"].clone();
    for i in 0..4 {
        add(&mut s, "primary", 2048 + i * 4096, 6143 + i * 4096);
    }
    assert_eq!(s.disks["/dev/vda"].mbr.as_ref().unwrap().entries.len(), 4);
    refuse(
        &mut s,
        "parted -s /dev/vda unit s mkpart primary ext4 24576s 28671s",
        Some(ErrorKind::Limit),
    );
    assert_eq!(s.execute("blkid /dev/vda1").unwrap().status, 2);
    s.execute("mkfs.ext4 -L sample /dev/vda1").unwrap();
    let report = s.execute("blkid /dev/vda1").unwrap();
    assert_eq!(report.status, 0);
    assert!(report.stdout.contains("TYPE=\"ext4\""));
    assert_eq!(s.disks["/dev/vdb"], protected);
    let report = s.report_disk("/dev/vda").unwrap().stdout;
    assert!(report.contains("SIMULATOR MBR"));
    assert!(report.contains("primary"));
    assert!(
        !report.contains("mbr1"),
        "Internal identifiers are not MBR names"
    );
    refuse(&mut s, "mkfs.ext4 /dev/vda1", Some(ErrorKind::Occupied));
}
#[test]
fn extended_container_is_not_a_filesystem_and_logical_payloads_are_real_separate_state() {
    let mut s = empty();
    add(&mut s, "primary", 2048, 6143);
    add(&mut s, "extended", 8192, 65535);
    add(&mut s, "logical", 10240, 18431);
    s.execute("parted -s /dev/vda unit s mkpart logical linux-swap 20480s 28671s")
        .unwrap();
    let entries = &s.disks["/dev/vda"].mbr.as_ref().unwrap().entries;
    assert_eq!(
        entries
            .iter()
            .map(|e| e.partition.number)
            .collect::<Vec<_>>(),
        vec![1, 2, 5, 6]
    );
    assert_eq!(entries[2].ebr, Some(8192));
    assert_eq!(entries[3].ebr, Some(8193));
    assert_eq!(entries[2].partition.content, disk::Content::Blank);
    refuse(&mut s, "mkfs.ext4 /dev/vda2", Some(ErrorKind::Invalid));
    refuse(&mut s, "mkswap /dev/vda2", Some(ErrorKind::Invalid));
    s.execute("mkfs -t ext4 -L reports /dev/vda5").unwrap();
    s.execute("mkswap -L scratch /dev/vda6").unwrap();
    let report = s.execute("blkid").unwrap().stdout;
    assert!(report.contains("/dev/vda5:"));
    assert!(report.contains("/dev/vda6:"));
    assert!(!report.contains("/dev/vda2:"));
    assert_eq!(
        s.execute("parted -s /dev/vda align-check optimal 5")
            .unwrap()
            .status,
        0
    );
    let report = s.report_disk("/dev/vda").unwrap().stdout;
    assert!(report.contains("extended-container"));
    assert!(report.contains("one-sector reservation"));
    assert_eq!(disk::State::from_json(&s.to_json().unwrap()).unwrap(), s);
}
#[test]
fn missing_container_illegal_overlap_and_missing_ebr_space_refuse_atomically() {
    let mut s = empty();
    refuse(
        &mut s,
        "parted -s /dev/vda unit s mkpart logical ext4 4096s 12287s",
        Some(ErrorKind::Invalid),
    );
    add(&mut s, "extended", 2048, 32767);
    for command in [
        "parted -s /dev/vda unit s mkpart logical ext4 2048s 4095s",
        "parted -s /dev/vda unit s mkpart logical ext4 4096s 65535s",
        "parted -s /dev/vda unit s mkpart primary ext4 4096s 8191s",
        "parted -s /dev/vda unit s mkpart extended 40960s 49151s",
    ] {
        refuse(&mut s, command, None);
    }
    add(&mut s, "logical", 2049, 4095);
    refuse(
        &mut s,
        "parted -s /dev/vda unit s mkpart logical ext4 4096s 6143s",
        Some(ErrorKind::Overlap),
    );
    add(&mut s, "logical", 4097, 6143);
    refuse(
        &mut s,
        "parted -s /dev/vda unit s mkpart logical ext4 3072s 7167s",
        Some(ErrorKind::Overlap),
    );
}
#[test]
fn supplied_use_flags_opaque_data_and_literal_command_boundary_still_protect_state() {
    let mut s = world();
    for path in [
        "/dev/vdc",
        "/dev/vdd",
        "/dev/vdz",
        "/dev/../vda",
        "/home/person/disk",
    ] {
        refuse(&mut s, &format!("parted -s {path} mklabel msdos"), None);
    }
    for command in [
        "parted -s /dev/vda mklabel msdos; cat /etc/passwd",
        "parted -s /dev/vda mklabel msdos $(id)",
        "parted -s /dev/vda mklabel msdos\n",
        "parted -s /dev/vda mklabel msdos --force",
    ] {
        refuse(&mut s, command, Some(ErrorKind::Unsupported));
    }
    s.disks.get_mut("/dev/vda").unwrap().occupied_without_table = true;
    refuse(
        &mut s,
        "parted -s /dev/vda mklabel msdos",
        Some(ErrorKind::Occupied),
    );
    s.disks.get_mut("/dev/vda").unwrap().occupied_without_table = false;
    s.disks.get_mut("/dev/vda").unwrap().in_use = true;
    refuse(
        &mut s,
        "parted -s /dev/vda mklabel msdos",
        Some(ErrorKind::Occupied),
    );
    s.disks.get_mut("/dev/vda").unwrap().in_use = false;
    s.execute("parted -s /dev/vda mklabel msdos").unwrap();
    refuse(
        &mut s,
        "parted -s /dev/vda mklabel gpt",
        Some(ErrorKind::Occupied),
    );
    refuse(
        &mut s,
        "parted -s /dev/vda mklabel msdos",
        Some(ErrorKind::Occupied),
    );
    for command in [
        "parted -s /dev/vda rm 1",
        "parted -s /dev/vda resizepart 1 100%",
        "parted -s /dev/vda unit s mkpart logical ext4 1MiB 2MiB",
    ] {
        refuse(&mut s, command, None);
    }
}
#[test]
fn malformed_profile_duplicate_ebr_and_cross_table_signatures_are_rejected() {
    let mut s = empty();
    add(&mut s, "extended", 2048, 65535);
    add(&mut s, "logical", 4096, 12287);
    add(&mut s, "logical", 16384, 24575);
    let good = serde_json::to_value(&s).unwrap();
    let mut candidates = Vec::new();
    let mut v = good.clone();
    v["mbr_profile"] = Value::Null;
    candidates.push(v);
    let mut v = good.clone();
    v["disks"]["/dev/vda"]["table"] = json!({"partitions":[]});
    candidates.push(v);
    let mut v = good.clone();
    v["disks"]["/dev/vda"]["mbr"]["entries"][2]["ebr"] = json!(2048);
    candidates.push(v);
    let mut v = good.clone();
    v["disks"]["/dev/vda"]["mbr"]["entries"][1]["partition"]["number"] = json!(4);
    candidates.push(v);
    let mut v = good.clone();
    v["disks"]["/dev/vda"]["mbr"]["entries"][1]["partition"]["name"] =
        json!("invented-on-disk-name");
    candidates.push(v);
    let mut v = good.clone();
    v["disks"]["/dev/vda"]["mbr"]["host_path"] = json!("/dev/sda");
    candidates.push(v);
    let mut v = good.clone();
    let row = v["disks"]["/dev/vda"]["mbr"]["entries"][0].clone();
    v["disks"]["/dev/vda"]["mbr"]["entries"] = Value::Array(vec![row; 17]);
    candidates.push(v);
    for v in candidates {
        assert!(disk::State::from_json(&v.to_string()).is_err());
    }
    assert!(disk::State::from_json(&" ".repeat(disk::MAX_STATE_BYTES + 1)).is_err());
    s.execute("mkfs.ext4 /dev/vda5").unwrap();
    let existing_uuid = s.disks["/dev/vdb"]
        .partitions()
        .find_map(|p| {
            if let disk::Content::Ext4 { uuid, .. } = &p.content {
                Some(uuid.clone())
            } else {
                None
            }
        })
        .unwrap();
    let mut v = serde_json::to_value(&s).unwrap();
    v["disks"]["/dev/vda"]["mbr"]["entries"][1]["partition"]["content"]["uuid"] =
        json!(existing_uuid);
    assert!(disk::State::from_json(&v.to_string()).is_err());
}
#[test]
fn mbr_uses_its_own_last_sector_and_refuses_beyond_the_disclosed_size_profile() {
    let mut s = empty();
    let end = s.disks["/dev/vda"].sectors - 1;
    add(&mut s, "primary", 2048, end);
    assert_eq!(
        s.disks["/dev/vda"].mbr.as_ref().unwrap().entries[0]
            .partition
            .end,
        end
    );
    let mut s = world();
    s.disks.get_mut("/dev/vda").unwrap().sectors = (1_u64 << 32) + 1;
    refuse(
        &mut s,
        "parted -s /dev/vda mklabel msdos",
        Some(ErrorKind::Limit),
    );
    s.disks.get_mut("/dev/vda").unwrap().sectors = 1_u64 << 32;
    s.execute("parted -s /dev/vda mklabel msdos").unwrap();
    add(&mut s, "extended", 2048, (1_u64 << 32) - 1);
    add(&mut s, "logical", 4096, 12287);
    assert_eq!(
        s.disks["/dev/vda"].mbr.as_ref().unwrap().entries[1].ebr,
        Some(2048)
    );
}
#[test]
fn bounded_logical_slots_refuse_without_sector_walking_or_partial_publication() {
    let mut s = empty();
    add(&mut s, "extended", 2048, 65535);
    for n in 0..12 {
        add(&mut s, "logical", 4096 + n * 4096, 6143 + n * 4096);
    }
    assert_eq!(
        s.disks["/dev/vda"]
            .mbr
            .as_ref()
            .unwrap()
            .entries
            .last()
            .unwrap()
            .partition
            .number,
        16
    );
    refuse(
        &mut s,
        "parted -s /dev/vda unit s mkpart logical ext4 57344s 59391s",
        Some(ErrorKind::Limit),
    );
}
#[test]
fn core_reset_requires_the_same_profile_and_restores_only_the_supplied_simulated_state() {
    let initial = world();
    let mut s = initial.clone();
    s.execute("parted -s /dev/vda mklabel msdos").unwrap();
    add(&mut s, "extended", 2048, 65535);
    add(&mut s, "logical", 4096, 12287);
    s.execute("mkfs.ext4 -L sample /dev/vda5").unwrap();
    let restored = disk::State::from_json(&s.to_json().unwrap()).unwrap();
    assert_eq!(restored, s);
    let before = s.clone();
    assert!(s.reset_from(&definition().initial).is_err());
    assert_eq!(s, before);
    s.reset_from(&initial).unwrap();
    assert_eq!(s, initial);
}
