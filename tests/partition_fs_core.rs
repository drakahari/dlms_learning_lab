//! Prospective pure-core tests. No host partition/filesystem commands or I/O.
use dlms_learning_lab::partition_fs::{
    self as fs, Action, Content, Disk, ErrorKind, File, Gpt, Partition, State, TypeHint,
};
use std::collections::BTreeMap;
fn disk() -> Disk {
    Disk {
        sectors: 32768,
        sector_bytes: 512,
        optimal_alignment: 2048,
        writable: true,
        in_use: false,
        occupied_without_table: false,
        table: None,
    }
}
fn fixture() -> State {
    let mut protected = disk();
    protected.table = Some(Gpt {
        partitions: vec![Partition {
            number: 1,
            name: "records".into(),
            start: 2048,
            end: 18431,
            type_hint: TypeHint::LinuxData,
            content: Content::Ext4 {
                label: "records".into(),
                uuid: "11111111-1111-4111-8111-111111111111".into(),
                blocks: 2048,
                files: BTreeMap::from([(
                    "/keep.txt".into(),
                    File {
                        text: "Retain this exact original data.\n".into(),
                    },
                )]),
            },
        }],
    });
    let mut opaque = disk();
    opaque.occupied_without_table = true;
    let mut readonly = disk();
    readonly.writable = false;
    State {
        version: 1,
        next_uuid: 1,
        disks: BTreeMap::from([
            ("/dev/vda".into(), disk()),
            ("/dev/vdb".into(), protected),
            ("/dev/vdc".into(), opaque),
            ("/dev/vdd".into(), readonly),
        ]),
    }
}
fn initialize(s: &mut State) {
    s.execute("parted -s /dev/vda mklabel gpt").unwrap();
}
fn add(s: &mut State, start: u64, end: u64) {
    s.apply(Action::AddPartition {
        disk: "/dev/vda".into(),
        name: "data".into(),
        start,
        end,
        type_hint: TypeHint::LinuxData,
    })
    .unwrap();
}
fn content(s: &State, n: usize) -> &Content {
    &s.disks["/dev/vda"].table.as_ref().unwrap().partitions[n].content
}
fn rejected(s: &mut State, command: &str, kind: Option<ErrorKind>) {
    let before = s.clone();
    let raw = s.to_json().unwrap();
    let e = s.execute(command).unwrap_err();
    if let Some(k) = kind {
        assert_eq!(e.kind, k, "{command}: {e}");
    }
    assert_eq!(&*s, &before, "{command}");
    assert_eq!(s.to_json().unwrap(), raw);
}

#[test]
fn allocating_does_not_format_and_all_reports_derive_from_current_state() {
    let mut s = fixture();
    initialize(&mut s);
    s.execute("parted -s /dev/vda unit s mkpart reports ext4 2048s 18431s")
        .unwrap();
    assert_eq!(content(&s, 0), &Content::Blank);
    assert_eq!(s.execute("blkid /dev/vda1").unwrap().status, 2);
    let r = s.execute("parted -s /dev/vda unit s print").unwrap();
    assert_eq!(r.status, 0);
    assert!(
        r.stdout
            .contains("1 2048 18431 16384 reports LinuxData none")
    );
    s.execute("mkfs.ext4 -b 4096 -L reports /dev/vda1").unwrap();
    assert!(matches!(content(&s,0),Content::Ext4{blocks:2048,label,..} if label=="reports"));
    let r = s.execute("blkid /dev/vda1").unwrap();
    assert_eq!(r.status, 0);
    assert!(r.stdout.contains("TYPE=\"ext4\""));
    assert!(r.stdout.contains("LABEL=\"reports\""));
    assert!(r.stdout.contains("00000000-0000-4000-8000-000000000001"));
    assert!(
        s.execute("parted -s /dev/vda unit s print")
            .unwrap()
            .stdout
            .contains("reports LinuxData ext4")
    );
}
#[test]
fn two_different_nonoverlapping_layouts_and_three_real_mkfs_spellings_work() {
    for (start, end) in [(2048, 18431), (4096, 20479)] {
        for command in [
            "mkfs.ext4 -b 4096 -L data /dev/vda1",
            "mkfs -t ext4 -L data -b 4096 /dev/vda1",
            "mke2fs -t ext4 -b 4096 -L data /dev/vda1",
        ] {
            let mut s = fixture();
            initialize(&mut s);
            add(&mut s, start, end);
            s.execute(command).unwrap();
            assert!(matches!(content(&s,0),Content::Ext4{blocks:2048,label,..} if label=="data"));
            assert_eq!(s.disks["/dev/vdb"], fixture().disks["/dev/vdb"]);
        }
    }
}
#[test]
fn swap_is_initialized_separately_and_does_not_claim_activation() {
    let mut s = fixture();
    initialize(&mut s);
    s.execute("parted -s /dev/vda unit s mkpart swap linux-swap 2048s 6143s")
        .unwrap();
    assert_eq!(content(&s, 0), &Content::Blank);
    s.execute("mkswap -L paging -U 22222222-2222-4222-8222-222222222222 /dev/vda1")
        .unwrap();
    assert!(matches!(content(&s,0),Content::Swap{usable_pages:511,label,..} if label=="paging"));
    assert!(
        s.execute("blkid /dev/vda1")
            .unwrap()
            .stdout
            .contains("TYPE=\"swap\"")
    );
    rejected(&mut s, "swapon /dev/vda1", Some(ErrorKind::Unsupported));
}
#[test]
fn gpt_primary_backup_boundaries_and_inclusive_adjacency_are_exact() {
    let mut s = fixture();
    initialize(&mut s);
    let last = s.disks["/dev/vda"].sectors - 34;
    add(&mut s, 34, 34);
    add(&mut s, 35, last);
    let t = s.disks["/dev/vda"].table.as_ref().unwrap();
    assert_eq!(t.partitions[0].bytes().unwrap(), 512);
    assert_eq!(t.partitions[1].end, last);
    assert_eq!(
        t.partitions.iter().map(|p| p.bytes().unwrap()).sum::<u64>(),
        (32768 - 67) * 512
    );
    for command in [
        "parted -s /dev/vda unit s mkpart bad ext4 33s 33s",
        "parted -s /dev/vda unit s mkpart bad ext4 32735s 32735s",
        "parted -s /dev/vda unit s mkpart bad ext4 100s 99s",
    ] {
        rejected(&mut s, command, Some(ErrorKind::Invalid));
    }
}
#[test]
fn every_overlap_shape_refuses_without_partial_changes_or_identity_consumption() {
    let mut s = fixture();
    initialize(&mut s);
    add(&mut s, 2048, 4095);
    for (start, end) in [
        (2048, 4095),
        (2047, 2048),
        (4095, 4096),
        (3000, 3500),
        (1024, 8191),
    ] {
        rejected(
            &mut s,
            &format!("parted -s /dev/vda unit s mkpart overlap ext4 {start}s {end}s"),
            Some(ErrorKind::Overlap),
        );
    }
    add(&mut s, 4096, 8191);
    assert_eq!(
        s.disks["/dev/vda"].table.as_ref().unwrap().partitions.len(),
        2
    );
}
#[test]
fn alignment_uses_supplied_start_geometry_without_rejecting_valid_unaligned_partition() {
    let mut s = fixture();
    initialize(&mut s);
    add(&mut s, 34, 2081);
    let before = s.clone();
    assert_eq!(
        s.execute("parted -s /dev/vda align-check optimal 1")
            .unwrap()
            .status,
        1
    );
    assert_eq!(s, before);
    add(&mut s, 4096, 6144);
    assert_eq!(
        s.execute("parted -s /dev/vda align-check optimal 2")
            .unwrap()
            .status,
        0
    ); // End need not align.
    s.disks.get_mut("/dev/vda").unwrap().optimal_alignment = 2;
    assert_eq!(
        s.execute("parted -s /dev/vda align-check optimal 1")
            .unwrap()
            .status,
        0
    );
}
#[test]
fn format_can_differ_from_type_hint_because_partition_code_is_not_a_filesystem() {
    let mut s = fixture();
    initialize(&mut s);
    add(&mut s, 2048, 6143);
    s.execute("mkswap /dev/vda1").unwrap();
    assert!(matches!(content(&s, 0), Content::Swap { .. }));
    assert_eq!(
        s.disks["/dev/vda"].table.as_ref().unwrap().partitions[0].type_hint,
        TypeHint::LinuxData
    );
}
#[test]
fn protected_sibling_data_opaque_disk_and_existing_signatures_never_get_erased() {
    let mut s = fixture();
    initialize(&mut s);
    add(&mut s, 2048, 18431);
    s.execute("mkfs.ext4 /dev/vda1").unwrap();
    for command in [
        "parted -s /dev/vda mklabel gpt",
        "parted -s /dev/vdb mklabel gpt",
        "parted -s /dev/vdc mklabel gpt",
        "mkfs.ext4 /dev/vdb1",
        "mkswap /dev/vdb1",
        "mkfs.ext4 /dev/vda1",
        "mkswap /dev/vda1",
    ] {
        rejected(&mut s, command, Some(ErrorKind::Occupied));
    }
    assert_eq!(s.disks["/dev/vdb"], fixture().disks["/dev/vdb"]);
    assert_eq!(s.disks["/dev/vdc"], fixture().disks["/dev/vdc"]);
}
#[test]
fn readonly_in_use_and_wrong_ids_preserve_whole_state() {
    let mut s = fixture();
    rejected(
        &mut s,
        "parted -s /dev/vdd mklabel gpt",
        Some(ErrorKind::Occupied),
    );
    s.disks.get_mut("/dev/vda").unwrap().in_use = true;
    rejected(
        &mut s,
        "parted -s /dev/vda mklabel gpt",
        Some(ErrorKind::Occupied),
    );
    s.disks.get_mut("/dev/vda").unwrap().in_use = false;
    initialize(&mut s);
    add(&mut s, 2048, 6143);
    for command in [
        "parted -s /dev/vdz mklabel gpt",
        "mkfs.ext4 /dev/vda2",
        "blkid /dev/vdz1",
    ] {
        rejected(&mut s, command, Some(ErrorKind::Unknown));
    }
    for command in [
        "mkfs.ext4 /dev/vda01",
        "mkfs.ext4 /dev/disk/by-id/a",
        "mkfs.ext4 /dev/vda",
        "mkfs.ext4 /dev/vda1/../vdb1",
        "mkfs.ext4 /dev/é1",
    ] {
        rejected(&mut s, command, None);
    }
}
#[test]
fn unsupported_real_tools_options_shell_and_units_do_not_get_canned_success() {
    let mut s = fixture();
    initialize(&mut s);
    add(&mut s, 2048, 6143);
    for command in [
        "fdisk /dev/vda",
        "gdisk /dev/vda",
        "parted -s /dev/vda mklabel msdos",
        "parted -s /dev/vda unit s mkpart primary logical ext4 8192s 12287s",
        "parted -s /dev/vda rm 1",
        "parted -s /dev/vda resizepart 1 8191s",
        "mkfs.xfs /dev/vda1",
        "mkfs.ext4 -F /dev/vda1",
        "mkfs.ext4 -b 1024 /dev/vda1",
        "mkfs.ext4 -L a -L b /dev/vda1",
        "parted -s /dev/vda unit s mkpart p ext4 1MiB 2MiB",
        "parted -s /dev/vda unit s mkpart p ext4 -2048s -1s",
        "mkfs.ext4 /dev/vda1; mkswap /dev/vda1",
        "mkfs.ext4 /dev/vda1\n",
        "mkfs.ext4 $(id)",
    ] {
        rejected(&mut s, command, None);
    }
}
#[test]
fn minimum_ext4_profile_swap_ten_page_threshold_and_partial_blocks_are_explicit() {
    let mut s = fixture();
    initialize(&mut s);
    add(&mut s, 34, 105); //9 pages.
    rejected(&mut s, "mkswap /dev/vda1", Some(ErrorKind::Invalid));
    rejected(&mut s, "mkfs.ext4 /dev/vda1", Some(ErrorKind::Unsupported));
    add(&mut s, 106, 185);
    s.execute("mkswap /dev/vda2").unwrap();
    assert!(matches!(
        content(&s, 1),
        Content::Swap {
            usable_pages: 9,
            ..
        }
    ));
    add(&mut s, 2048, 4096);
    s.execute("mkfs.ext4 /dev/vda3").unwrap();
    assert!(matches!(content(&s, 2), Content::Ext4 { blocks: 256, .. })); //Trailing512B not claimed as a block.
}
#[test]
fn explicit_uuid_collision_refuses_and_generated_ids_skip_used_values_durably() {
    let mut s = fixture();
    initialize(&mut s);
    add(&mut s, 2048, 6143);
    add(&mut s, 6144, 10239);
    add(&mut s, 10240, 14335);
    rejected(
        &mut s,
        "mkfs.ext4 -U 11111111-1111-4111-8111-111111111111 /dev/vda1",
        Some(ErrorKind::Occupied),
    );
    s.execute("mkfs.ext4 -U 00000000-0000-4000-8000-000000000001 /dev/vda1")
        .unwrap();
    assert_eq!(s.next_uuid, 1);
    s.execute("mkswap /dev/vda2").unwrap();
    assert_eq!(s.next_uuid, 3);
    let mut resumed = State::from_json(&s.to_json().unwrap()).unwrap();
    resumed.execute("mkfs.ext4 /dev/vda3").unwrap();
    assert_eq!(resumed.next_uuid, 4);
    assert!(
        resumed
            .execute("blkid /dev/vda3")
            .unwrap()
            .stdout
            .contains("000000000003")
    );
}
#[test]
fn slot_limit_accepts_sixteen_then_refuses_seventeenth_before_mutation() {
    let mut s = fixture();
    initialize(&mut s);
    for n in 0..16 {
        add(&mut s, 34 + n * 8, 41 + n * 8);
    }
    rejected(
        &mut s,
        "parted -s /dev/vda unit s mkpart excess ext4 2048s 4095s",
        Some(ErrorKind::Limit),
    );
    assert_eq!(
        s.disks["/dev/vda"]
            .table
            .as_ref()
            .unwrap()
            .partitions
            .iter()
            .map(|p| p.number)
            .collect::<Vec<_>>(),
        (1..=16).collect::<Vec<_>>()
    );
}
#[test]
fn arithmetic_and_bad_geometry_are_refused_without_capacity_sized_allocations() {
    let mut s = fixture();
    initialize(&mut s);
    rejected(
        &mut s,
        "parted -s /dev/vda unit s mkpart overflow ext4 34s 18446744073709551615s",
        Some(ErrorKind::Invalid),
    );
    rejected(
        &mut s,
        "parted -s /dev/vda unit s mkpart overflow ext4 18446744073709551616s 18446744073709551617s",
        Some(ErrorKind::Invalid),
    );
    for (sectors, sector_bytes, alignment) in [
        (127, 512, 2048),
        (fs::MAX_SECTORS + 1, 512, 2048),
        (32768, 4096, 2048),
        (32768, 512, 3),
        (32768, 512, 0),
        (32768, 512, 16384),
    ] {
        let mut bad = fixture();
        let d = bad.disks.get_mut("/dev/vda").unwrap();
        d.sectors = sectors;
        d.sector_bytes = sector_bytes;
        d.optimal_alignment = alignment;
        assert!(bad.validate().is_err());
    }
    let mut maximum = fixture();
    maximum.disks.get_mut("/dev/vda").unwrap().sectors = fs::MAX_SECTORS;
    maximum.validate().unwrap();
    initialize(&mut maximum);
    add(&mut maximum, 34, fs::MAX_SECTORS - 34);
    assert!(maximum.to_json().unwrap().len() < 5000);
}
#[test]
fn reset_resume_preserve_full_state_and_different_supplied_environment_cannot_reset() {
    let initial = fixture();
    let mut s = initial.clone();
    initialize(&mut s);
    add(&mut s, 2048, 6143);
    s.execute("mkfs.ext4 -L reports /dev/vda1").unwrap();
    let raw = s.to_json().unwrap();
    let restored = State::from_json(&raw).unwrap();
    assert_eq!(restored, s);
    assert_eq!(restored.to_json().unwrap(), raw);
    s.reset_from(&initial).unwrap();
    assert_eq!(s, initial);
    let mut changed = initial.clone();
    changed.disks.get_mut("/dev/vda").unwrap().sectors += 1;
    assert!(!s.environment_matches(&changed));
    let before = s.clone();
    assert!(s.reset_from(&changed).is_err());
    assert_eq!(s, before);
}
#[test]
fn codec_rejects_raw_bounds_duplicate_keys_unknown_fields_corruption_and_excess_poison() {
    let s = fixture();
    let raw = s.to_json().unwrap();
    assert!(State::from_json(&" ".repeat(fs::MAX_STATE_BYTES + 1)).is_err());
    assert!(
        State::from_json(&raw.replace("\"version\":1", "\"version\":1,\"version\":1")).is_err()
    );
    assert!(
        State::from_json(&raw.replacen("\"disks\":{", "\"disks\":{\"/dev/vda\":{},", 1)).is_err()
    );
    assert!(State::from_json(&raw.replacen('{', "{\"extra\":0,", 1)).is_err());
    assert!(State::from_json(&raw[..raw.len() - 1]).is_err());
    let disk_json = serde_json::to_string(&disk()).unwrap();
    let prefix = format!(
        "{{\"version\":1,\"next_uuid\":1,\"disks\":{{\"/dev/vda\":{disk_json},\"/dev/vdb\":{disk_json},\"/dev/vdc\":{disk_json},\"/dev/vdd\":{disk_json},\"/dev/vde\":"
    );
    let e = serde_json::from_str::<State>(&format!("{prefix}{{\"poison\": ["))
        .unwrap_err()
        .to_string();
    assert!(e.contains("count/duplicate before value"), "{e}");
    let duplicate = format!(
        "{{\"version\":1,\"next_uuid\":1,\"disks\":{{\"/dev/vda\":{disk_json},\"/dev/vda\":{{\"poison\": ["
    );
    assert!(
        serde_json::from_str::<State>(&duplicate)
            .unwrap_err()
            .to_string()
            .contains("count/duplicate before value")
    );
    let mut p = fixture();
    initialize(&mut p);
    for n in 0..16 {
        add(&mut p, 34 + n * 8, 41 + n * 8);
    }
    let mut value = serde_json::to_value(&p).unwrap();
    value["disks"]["/dev/vda"]["table"]["partitions"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"poison":true}));
    assert!(
        serde_json::from_value::<State>(value)
            .unwrap_err()
            .to_string()
            .contains("bound before excess body")
    );
}
#[test]
fn bounded_text_files_aggregate_and_escaped_wire_bytes_are_checked_before_clone() {
    let initial = fixture();
    let mut bad = initial.clone();
    let content = &mut bad
        .disks
        .get_mut("/dev/vdb")
        .unwrap()
        .table
        .as_mut()
        .unwrap()
        .partitions[0]
        .content;
    if let Content::Ext4 { files, .. } = content {
        files.get_mut("/keep.txt").unwrap().text = "x".repeat(fs::MAX_FILE_BYTES + 1);
    } else {
        unreachable!()
    }
    assert!(bad.validate().is_err());
    let before = bad.clone();
    assert!(
        bad.apply(Action::CreateGpt {
            disk: "/dev/vda".into()
        })
        .is_err()
    );
    assert_eq!(bad, before);
    let mut value = serde_json::to_value(&initial).unwrap();
    value["disks"]["/dev/vdb"]["table"]["partitions"][0]["name"] =
        serde_json::json!("x".repeat(37));
    assert!(State::from_json(&value.to_string()).is_err());
    let mut escaped = initial.clone();
    if let Content::Ext4 { files, .. } = &mut escaped
        .disks
        .get_mut("/dev/vdb")
        .unwrap()
        .table
        .as_mut()
        .unwrap()
        .partitions[0]
        .content
    {
        files.clear();
        for n in 0..8 {
            files.insert(
                format!("/f{n}"),
                File {
                    text: "\u{0001}".repeat(fs::MAX_FILE_BYTES),
                },
            );
        }
    }
    assert!(escaped.validate().unwrap_err().message.contains("Escaped"));
    let mut total = initial.clone();
    if let Content::Ext4 { files, .. } = &mut total
        .disks
        .get_mut("/dev/vdb")
        .unwrap()
        .table
        .as_mut()
        .unwrap()
        .partitions[0]
        .content
    {
        files.clear();
        for n in 0..9 {
            files.insert(
                format!("/f{n}"),
                File {
                    text: "x".repeat(fs::MAX_FILE_BYTES),
                },
            );
        }
    }
    assert!(
        total
            .validate()
            .unwrap_err()
            .message
            .contains("Retained file bytes")
    );
}
#[test]
fn invalid_typed_actions_cannot_bypass_parser_limits_or_change_supplied_facts() {
    let mut s = fixture();
    initialize(&mut s);
    add(&mut s, 2048, 6143);
    for a in [
        Action::FormatExt4 {
            partition: "/dev/vda1".into(),
            label: "x".repeat(17),
            uuid: None,
        },
        Action::InitializeSwap {
            partition: "/dev/vda1".into(),
            label: "ok".into(),
            uuid: Some("NOT-A-UUID".into()),
        },
        Action::AddPartition {
            disk: "/dev/vda".into(),
            name: "x".repeat(37),
            start: 8192,
            end: 12287,
            type_hint: TypeHint::LinuxData,
        },
    ] {
        let before = s.clone();
        assert!(s.apply(a).is_err());
        assert_eq!(s, before);
    }
    assert!(s.environment_matches(&fixture()));
    rejected(
        &mut s,
        &"a".repeat(fs::MAX_COMMAND_BYTES + 1),
        Some(ErrorKind::Unsupported),
    );
}

#[test]
fn environment_comparison_refuses_invalid_self_and_source_before_comparing_facts() {
    let valid = fixture();
    valid.validate().unwrap();
    assert!(valid.environment_matches(&valid));
    let mut prepared = valid.clone();
    initialize(&mut prepared);
    add(&mut prepared, 2048, 6143);
    prepared.execute("mkfs.ext4 /dev/vda1").unwrap();
    assert!(prepared.environment_matches(&valid));
    assert!(valid.environment_matches(&prepared));
    let mut invalid = Vec::new();
    let mut version = valid.clone();
    version.version += 1;
    invalid.push(version);
    let mut counter = valid.clone();
    counter.next_uuid = 0;
    invalid.push(counter);
    let mut range = valid.clone();
    range
        .disks
        .get_mut("/dev/vdb")
        .unwrap()
        .table
        .as_mut()
        .unwrap()
        .partitions[0]
        .start = 33;
    invalid.push(range);
    let mut overlap = valid.clone();
    let table = overlap
        .disks
        .get_mut("/dev/vdb")
        .unwrap()
        .table
        .as_mut()
        .unwrap();
    let mut second = table.partitions[0].clone();
    second.number = 2;
    second.name = "overlap".into();
    second.content = Content::Blank;
    table.partitions.push(second);
    assert_eq!(overlap.validate().unwrap_err().kind, ErrorKind::Overlap);
    invalid.push(overlap);
    let mut signature = valid.clone();
    if let Content::Ext4 { blocks, .. } = &mut signature
        .disks
        .get_mut("/dev/vdb")
        .unwrap()
        .table
        .as_mut()
        .unwrap()
        .partitions[0]
        .content
    {
        *blocks += 1;
    } else {
        unreachable!();
    }
    invalid.push(signature);
    for bad in invalid {
        assert!(bad.validate().is_err());
        assert!(!bad.environment_matches(&valid));
        assert!(!valid.environment_matches(&bad));
        assert!(!bad.environment_matches(&bad));
        let before = prepared.clone();
        assert!(prepared.reset_from(&bad).is_err());
        assert_eq!(prepared, before);
    }
}
