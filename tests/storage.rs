use dlms_learning_lab::storage::{
    self, ErrorKind, Filesystem, Inventory, Mount, MountOutcome, SessionOutcome, State, Volume,
};
use std::collections::BTreeMap;

const DATA: &str = "11111111-1111-4111-8111-111111111111";
const ARCHIVE: &str = "22222222-2222-4222-8222-222222222222";
const SPARE: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
fn aliases() -> BTreeMap<String, String> {
    [
        ("/dev/sdb1", "data"),
        ("/dev/sdc1", "archive"),
        ("/dev/sdd1", "spare"),
    ]
    .into_iter()
    .map(|(a, b)| (a.into(), b.into()))
    .collect()
}
fn swapped() -> BTreeMap<String, String> {
    [
        ("/dev/sdc1", "data"),
        ("/dev/sdd1", "archive"),
        ("/dev/sdb1", "spare"),
    ]
    .into_iter()
    .map(|(a, b)| (a.into(), b.into()))
    .collect()
}
fn inventory() -> Inventory {
    Inventory::new(
        [("data", DATA), ("archive", ARCHIVE), ("spare", SPARE)]
            .into_iter()
            .map(|(id, uuid)| {
                (
                    id.into(),
                    Volume {
                        uuid: uuid.into(),
                        filesystem: Filesystem::Ext4,
                    },
                )
            })
            .collect(),
        aliases(),
        vec!["/data".into(), "/archive".into(), "/maintenance".into()],
    )
    .unwrap()
}
fn configuration() -> String {
    format!(
        "# A saved table, not the current mount table\nUUID={DATA} /data ext4 defaults 0 2\nUUID={ARCHIVE} /archive ext4 ro,auto 0 2\nUUID={SPARE} /maintenance ext4 rw,noauto 0 0\n"
    )
}
fn state() -> State {
    State::new(inventory(), configuration(), BTreeMap::new()).unwrap()
}
fn mounted(volume: &str, read_only: bool) -> Mount {
    Mount {
        volume: volume.into(),
        read_only,
    }
}

#[test]
fn unique_bijective_inventory_and_nonnested_points_are_required() {
    let inv = inventory();
    assert_eq!(inv.device_for("archive"), Some("/dev/sdc1"));
    for bad in [
        vec!["/data", "/data/child"],
        vec!["/data", "/data"],
        vec!["/"],
        vec!["/data/../etc"],
        vec!["relative"],
    ] {
        assert!(
            Inventory::new(
                inv.volumes().clone(),
                aliases(),
                bad.iter().map(|s| s.to_string()).collect()
            )
            .is_err()
        );
    }
    let mut bad = aliases();
    bad.insert("/dev/sdc1".into(), "data".into());
    assert!(Inventory::new(inv.volumes().clone(), bad, vec!["/data".into()]).is_err());
    let mut volumes = inv.volumes().clone();
    volumes.get_mut("archive").unwrap().uuid = DATA.into();
    assert!(Inventory::new(volumes, aliases(), vec!["/data".into()]).is_err());
    let mut bad = aliases();
    bad.remove("/dev/sdb1");
    bad.insert("/dev/../etc".into(), "data".into());
    assert!(Inventory::new(inv.volumes().clone(), bad, vec!["/data".into()]).is_err());
}

#[test]
fn parser_keeps_explicit_options_across_defaults_and_uses_last_conflicting_option() {
    for (options, ro, auto) in [
        ("defaults", false, true),
        ("ro,defaults", true, true),
        ("noauto,defaults", false, false),
        ("ro,rw,noauto,auto", false, true),
        ("rw,ro,auto,noauto", true, false),
    ] {
        let e = storage::parse(&format!("UUID={DATA} /data ext4 {options} 7 3\n"))
            .unwrap()
            .remove(0);
        assert_eq!((e.read_only, e.automatic, e.dump, e.pass), (ro, auto, 7, 3));
    }
    let e = storage::parse(&format!(
        "# explanation\r\n\tUUID={SPARE} /maintenance xfs ro 0 0\r\n\n"
    ))
    .unwrap();
    assert_eq!(e[0].line, 2);
    assert_eq!(e[0].filesystem, Filesystem::Xfs);
    assert!(storage::parse("# only a comment\n\n").unwrap().is_empty());
}

#[test]
fn uuid_identity_uses_exact_inventory_spelling_and_blkid_identifies_distinct_volumes() {
    let mut s = state();
    s.save_configuration(format!(
        "UUID={} /maintenance ext4 rw 0 0",
        SPARE.to_ascii_uppercase()
    ))
    .unwrap();
    assert_eq!(
        s.mount("/maintenance").unwrap_err().kind,
        ErrorKind::UnknownSource
    );
    assert_eq!(s.validate_configuration()[0].kind, ErrorKind::UnknownSource);
    s.save_configuration(format!("UUID={SPARE} /maintenance ext4 rw 0 0"))
        .unwrap();
    s.mount("/maintenance").unwrap();
    assert_eq!(s.live()["/maintenance"].volume, "spare");
    let listing = s.inventory().blkid();
    assert_eq!(listing.lines().count(), 3);
    assert!(listing.contains(&format!("/dev/sdd1: UUID=\"{SPARE}\" TYPE=\"ext4\"")));
    assert_eq!(
        s.mount_listing(),
        "/dev/sdd1 on /maintenance type ext4 (rw)"
    );
}

#[test]
fn malformed_and_unimplemented_fstab_forms_are_honest_diagnostics() {
    for (text, kind) in [
        ("/dev/sdb1 /data ext4 defaults", ErrorKind::Invalid),
        ("UUID=bad /data ext4 defaults 0 2", ErrorKind::Invalid),
        ("LABEL=data /data ext4 defaults 0 2", ErrorKind::Unsupported),
        ("/dev/sdb1 /data btrfs defaults 0 2", ErrorKind::Unsupported),
        ("/dev/sdb1 /data ext4 nosuid 0 2", ErrorKind::Unsupported),
        ("/dev/sdb1 /data ext4 rw, 0 2", ErrorKind::Unsupported),
        ("/dev/sdb1 /data ext4 rw 0 -1", ErrorKind::Invalid),
        ("/dev/sdb1 /data ext4 rw +1 2", ErrorKind::Invalid),
        ("/dev/sdb1 /data ext4 rw 0 4294967296", ErrorKind::Invalid),
        (
            "/dev/sdb1 /data ext4 rw 0 2 # comment",
            ErrorKind::Unsupported,
        ),
        (
            "/dev/sdb1 /data\\040disk ext4 rw 0 2",
            ErrorKind::Unsupported,
        ),
        ("'/dev/sdb1' /data ext4 rw 0 2", ErrorKind::Unsupported),
        ("/dev/sdb1 /data/../archive ext4 rw 0 2", ErrorKind::Invalid),
    ] {
        let e = storage::parse(text).unwrap_err();
        assert_eq!(e.kind, kind, "{text}: {e}");
        assert_eq!(e.line, Some(1));
    }
}

#[test]
fn validate_reports_resolution_type_and_fixed_directory_mistakes_without_mutation() {
    let mut s = state();
    s.save_configuration(format!("UUID=99999999-9999-4999-8999-999999999999 /data ext4 rw 0 2\nUUID={ARCHIVE} /archive xfs ro 0 2\nUUID={SPARE} /unknown ext4 rw 0 2")).unwrap();
    let before = s.clone();
    let errors = s.validate_configuration();
    assert_eq!(
        errors.iter().map(|e| e.kind).collect::<Vec<_>>(),
        vec![
            ErrorKind::UnknownSource,
            ErrorKind::TypeMismatch,
            ErrorKind::Invalid
        ]
    );
    assert_eq!(s, before);
    assert_eq!(s.mount("/data").unwrap_err().kind, ErrorKind::UnknownSource);
    assert_eq!(
        s.mount("/archive").unwrap_err().kind,
        ErrorKind::TypeMismatch
    );
    assert_eq!(s, before);
}

#[test]
fn saved_configuration_and_live_policy_are_independent_until_explicit_remount_work() {
    let mut s = state();
    assert!(s.validate_configuration().is_empty());
    assert_eq!(s.mount("/data").unwrap(), MountOutcome::Mounted);
    assert_eq!(s.mount("/data").unwrap(), MountOutcome::AlreadyMounted);
    let ro = configuration().replace("/data ext4 defaults", "/data ext4 ro");
    s.save_configuration(ro).unwrap();
    assert!(!s.live()["/data"].read_only);
    let before = s.clone();
    assert_eq!(s.mount("/data").unwrap_err().kind, ErrorKind::Occupied);
    assert_eq!(s, before);
    s.umount("/data").unwrap();
    s.mount("/data").unwrap();
    assert!(s.live()["/data"].read_only);
    assert_eq!(
        s.umount("/archive").unwrap_err().kind,
        ErrorKind::NotMounted
    );
}

#[test]
fn automatic_selection_skips_noauto_but_an_explicit_mount_still_works() {
    let s = state();
    let fresh = s.fresh_session(aliases()).unwrap();
    assert_eq!(fresh.state.live().len(), 2);
    assert_eq!(
        fresh.outcomes[2],
        SessionOutcome::Manual {
            target: "/maintenance".into()
        }
    );
    let mut tested = fresh.state;
    tested.mount("/maintenance").unwrap();
    assert_eq!(tested.live()["/maintenance"], mounted("spare", false));
    assert!(s.live().is_empty());
}

#[test]
fn skipped_manual_is_not_a_validity_claim_and_session_aliases_must_be_a_permutation() {
    let mut s = state();
    s.save_configuration(configuration().replace(SPARE, "99999999-9999-4999-8999-999999999999"))
        .unwrap();
    let fresh = s.fresh_session(aliases()).unwrap();
    assert!(matches!(fresh.outcomes[2], SessionOutcome::Manual { .. }));
    assert_eq!(s.validate_configuration()[0].kind, ErrorKind::UnknownSource);
    let mut different = aliases();
    different.remove("/dev/sdb1");
    different.insert("/dev/new".into(), "data".into());
    assert!(s.fresh_session(different).is_err());
}

#[test]
fn uuid_policy_survives_alias_permutation_without_changing_original_session() {
    let mut s = state();
    s.mount("/maintenance").unwrap();
    let before = s.clone();
    let fresh = s.fresh_session(swapped()).unwrap();
    assert_eq!(fresh.state.live()["/data"], mounted("data", false));
    assert_eq!(fresh.state.live()["/archive"], mounted("archive", true));
    assert!(!fresh.state.live().contains_key("/maintenance"));
    assert_eq!(
        fresh.state.inventory().device_for("data"),
        Some("/dev/sdc1")
    );
    assert_eq!(s, before);
}

#[test]
fn a_live_device_path_can_mount_the_wrong_equal_type_volume_after_renumbering() {
    let mut s = state();
    s.save_configuration("/dev/sdb1 /data ext4 rw 0 2\n/dev/sdc1 /archive ext4 ro 0 2".into())
        .unwrap();
    assert!(s.validate_configuration().is_empty());
    s.mount("/data").unwrap();
    assert_eq!(s.live()["/data"].volume, "data");
    let fresh = s.fresh_session(swapped()).unwrap();
    assert_eq!(fresh.state.live()["/data"].volume, "spare");
    assert_ne!(fresh.state.live()["/archive"].volume, "archive");
}

#[test]
fn fresh_session_continues_independent_entries_after_a_missing_source() {
    let mut s = state();
    s.save_configuration(configuration().replace(DATA, "99999999-9999-4999-8999-999999999999"))
        .unwrap();
    let fresh = s.fresh_session(swapped()).unwrap();
    assert!(
        matches!(fresh.outcomes[0],SessionOutcome::Failed{ref error,..} if error.kind==ErrorKind::UnknownSource)
    );
    assert!(matches!(fresh.outcomes[1], SessionOutcome::Mounted { .. }));
    assert_eq!(fresh.state.live()["/archive"].volume, "archive");
    assert!(!fresh.state.live().contains_key("/data"));
}

#[test]
fn alias_equivalent_duplicate_volume_sources_cannot_bypass_single_mount_scope() {
    let mut s = state();
    s.save_configuration(format!(
        "UUID={DATA} /data ext4 rw 0 2\n/dev/sdb1 /archive ext4 ro 0 2"
    ))
    .unwrap();
    let errors = s.validate_configuration();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, ErrorKind::Unsupported);
    let fresh = s.fresh_session(aliases()).unwrap();
    assert!(matches!(fresh.outcomes[0], SessionOutcome::Mounted { .. }));
    assert!(
        matches!(fresh.outcomes[1],SessionOutcome::Failed{ref error,..} if error.kind==ErrorKind::Occupied)
    );
    assert_eq!(fresh.state.live().len(), 1);
    s.mount("/data").unwrap();
    let before = s.clone();
    assert!(s.mount("/archive").is_err());
    assert_eq!(s, before);
}

#[test]
fn parser_duplicate_targets_and_structural_errors_block_typed_actions_atomically() {
    let mut s = state();
    s.save_configuration(format!(
        "UUID={DATA} /data ext4 rw 0 2\nUUID={ARCHIVE} /data ext4 ro 0 2"
    ))
    .unwrap();
    let before = s.clone();
    assert_eq!(s.validate_configuration()[0].kind, ErrorKind::Invalid);
    assert!(s.mount("/data").is_err());
    assert!(s.fresh_session(aliases()).is_err());
    assert_eq!(s, before);
    s.save_configuration("not a supported table".into())
        .unwrap();
    let before = s.clone();
    assert!(s.mount("/data").is_err());
    assert_eq!(s, before);
}

#[test]
fn current_live_success_can_be_broken_again_without_any_sticky_core_completion() {
    let mut s = state();
    let success = |s: &State| {
        s.live().get("/data") == Some(&mounted("data", false))
            && s.validate_configuration().is_empty()
    };
    assert!(!success(&s));
    s.mount("/data").unwrap();
    assert!(success(&s));
    s.umount("/data").unwrap();
    assert!(!success(&s));
    s.mount("/data").unwrap();
    s.save_configuration("broken".into()).unwrap();
    assert!(!success(&s));
}

#[test]
fn validated_snapshots_reject_unknown_identity_duplicate_keys_and_bad_live_mounts() {
    let mut s = state();
    s.mount("/data").unwrap();
    let encoded = serde_json::to_string_pretty(&s).unwrap();
    assert_eq!(State::from_json(&encoded).unwrap(), s);
    for bad in [
        encoded.replace("\"volume\": \"data\"", "\"volume\": \"missing\""),
        encoded.replace("\"/data\": {", "\"/unknown\": {"),
        encoded.replace("\"read_only\": false", "\"read_only\": false, \"extra\": 1"),
    ] {
        assert!(State::from_json(&bad).is_err(), "{bad}");
    }
    let raw = serde_json::to_value(&s).unwrap();
    let mut duplicate = raw.clone();
    duplicate["live"]["/archive"] = duplicate["live"]["/data"].clone();
    assert!(serde_json::from_value::<State>(duplicate).is_err());
    let mut points = raw.clone();
    points["inventory"]["mountpoints"] = serde_json::json!(["/data", "/data"]);
    assert!(serde_json::from_value::<State>(points).is_err());
    let compact = serde_json::to_string(&s).unwrap();
    let duplicate = compact.replace(
        "\"configuration\":",
        "\"configuration\":\"\",\"configuration\":",
    );
    assert!(State::from_json(&duplicate).is_err());
    let mut changed = raw;
    changed["inventory"]["volumes"]["data"]["uuid"] =
        serde_json::json!("99999999-9999-4999-8999-999999999999");
    let other = serde_json::from_value::<State>(changed).unwrap();
    assert!(!s.same_inventory(&other)); // Future progress loader must pin this comparison to the authored initial state.
}

#[test]
fn bounded_text_save_failure_preserves_prior_state_and_worst_case_state_roundtrips() {
    let mut s = state();
    let before = s.clone();
    for text in [
        "x".repeat(storage::MAX_CONFIG_BYTES + 1),
        "\n".repeat(129),
        "\u{1b}[31m".into(),
    ] {
        assert!(s.save_configuration(text).is_err());
        assert_eq!(s, before);
    }
    // A full-size invalid draft is legitimate saved work, not a valid mount plan.
    s.save_configuration("\\".repeat(storage::MAX_CONFIG_BYTES))
        .unwrap();
    let raw = serde_json::to_string_pretty(&s).unwrap();
    assert!(raw.len() < storage::MAX_STATE_BYTES);
    assert_eq!(State::from_json(&raw).unwrap(), s);
    assert!(State::from_json(&" ".repeat(storage::MAX_STATE_BYTES + 1)).is_err());
}

#[test]
fn all_eight_volumes_and_mountpoints_work_and_ninth_is_rejected() {
    let mut volumes = BTreeMap::new();
    let mut devices = BTreeMap::new();
    let mut points = Vec::new();
    let mut text = String::new();
    for i in 0..8 {
        let id = format!("volume{i}");
        let uuid = format!("00000000-0000-4000-8000-{i:012}");
        let p = format!("/vol{i}");
        devices.insert(format!("/dev/disk{i}"), id.clone());
        volumes.insert(
            id,
            Volume {
                uuid: uuid.clone(),
                filesystem: Filesystem::Xfs,
            },
        );
        points.push(p.clone());
        text.push_str(&format!("UUID={uuid} {p} xfs rw 0 0\n"));
    }
    let inv = Inventory::new(volumes.clone(), devices.clone(), points.clone()).unwrap();
    let s = State::new(inv, text, BTreeMap::new()).unwrap();
    let fresh = s.fresh_session(devices.clone()).unwrap();
    assert_eq!(fresh.state.live().len(), 8);
    let raw = serde_json::to_string(&fresh.state).unwrap();
    assert_eq!(State::from_json(&raw).unwrap(), fresh.state);
    volumes.insert(
        "ninth".into(),
        Volume {
            uuid: "99999999-9999-4999-8999-999999999999".into(),
            filesystem: Filesystem::Ext4,
        },
    );
    devices.insert("/dev/ninth".into(), "ninth".into());
    points.push("/ninth".into());
    assert!(Inventory::new(volumes, devices, points).is_err());
}
