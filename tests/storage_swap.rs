//! Pure finite swap proof. No host swapon, memory changes or curriculum credit.
use dlms_learning_lab::storage::{self, Source, swap};
use std::collections::BTreeMap;
fn inventory() -> swap::Inventory {
    swap::Inventory {
        areas: (0..4)
            .map(|n| swap::Area {
                id: format!("area{n}"),
                device: format!("/dev/vd{}1", char::from(b'b' + n)),
                uuid: format!("00000000-0000-0000-0000-00000000000{n}"),
                size_bytes: 4096 * (u64::from(n) + 1),
                prepared: n != 2,
                available: n != 3,
            })
            .collect(),
    }
}
fn uuid(n: usize) -> String {
    format!("UUID=00000000-0000-0000-0000-00000000000{n}")
}
fn row(n: usize, opts: &str) -> String {
    format!("{} none swap {opts} 0 0\n", uuid(n))
}
#[test]
fn saved_policy_and_active_priority_are_distinct_and_noauto_is_not_a_ban() {
    let f = inventory();
    f.validate().unwrap();
    let mut active = BTreeMap::new();
    let text = row(0, "pri=20,defaults,auto") + &row(1, "noauto,pri=80");
    let results = swap::activate_all(&f, &mut active, &text).unwrap();
    assert_eq!(
        results,
        vec![
            swap::Outcome::Activated { id: "area0".into() },
            swap::Outcome::Skipped { id: "area1".into() }
        ]
    );
    assert_eq!(active, BTreeMap::from([("area0".into(), 20)]));
    let changed = row(0, "pri=100") + &row(1, "noauto,pri=80");
    swap::activate_all(&f, &mut active, &changed).unwrap();
    assert_eq!(
        active["area0"], 20,
        "bulk activation skips an existing active area"
    );
    let source = swap::source(&uuid(0)).unwrap();
    let before = active.clone();
    assert!(
        swap::activate(&f, &mut active, &source, 100, false)
            .unwrap_err()
            .to_string()
            .contains("already active")
    );
    assert_eq!(active, before);
    swap::deactivate(&f, &mut active, &source).unwrap();
    assert!(swap::activate(&f, &mut active, &source, 100, false).unwrap());
    assert!(
        swap::activate(
            &f,
            &mut active,
            &Source::Device("/dev/vdc1".into()),
            80,
            false
        )
        .unwrap()
    );
    assert_eq!(
        active,
        BTreeMap::from([("area0".into(), 100), ("area1".into(), 80)])
    );
    assert!(
        swap::listing(&f, &active)
            .unwrap()
            .contains("/dev/vdb1 partition 4096 0 100")
    );
}
#[test]
fn bulk_activation_keeps_successes_and_explains_each_independent_failure() {
    let f = inventory();
    let mut active = BTreeMap::new();
    let text = row(0, "pri=0") + &row(2, "pri=32767") + &row(1, "pri=10") + &row(3, "pri=90");
    let outcomes = swap::activate_all(&f, &mut active, &text).unwrap();
    assert_eq!(outcomes.len(), 4);
    assert!(matches!(&outcomes[0],swap::Outcome::Activated{id}if id=="area0"));
    assert!(
        matches!(&outcomes[1],swap::Outcome::Failed{line:2,reason}if reason.contains("signature"))
    );
    assert!(matches!(&outcomes[2],swap::Outcome::Activated{id}if id=="area1"));
    assert!(
        matches!(&outcomes[3],swap::Outcome::Failed{line:4,reason}if reason.contains("unavailable"))
    );
    assert_eq!(
        active,
        BTreeMap::from([("area0".into(), 0), ("area1".into(), 10)])
    );
    let unknown = "UUID=99999999-9999-9999-9999-999999999999 none swap pri=15 0 0\n";
    let mut empty = BTreeMap::new();
    let out = swap::activate_all(
        &f,
        &mut empty,
        &(row(0, "pri=0") + unknown + &row(1, "pri=1")),
    )
    .unwrap();
    assert!(
        matches!(&out[1],swap::Outcome::Failed{line:2,reason}if reason.contains("No supplied"))
    );
    assert_eq!(
        empty.len(),
        2,
        "unknown row does not undo or hide other successes"
    );
}
#[test]
fn typed_policy_parser_preserves_mount_rows_and_line_numbers_without_weakening_legacy() {
    let mount = "UUID=aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa /data ext4 defaults 0 2\n";
    let text = format!("# policy\n{}\n{mount}", row(0, "defaults,pri=20"));
    assert!(
        storage::parse(&text).is_err(),
        "old external parser remains mount-only"
    );
    let t = swap::table(&text).unwrap();
    assert_eq!(t.swaps[0].line, 2);
    assert_eq!(t.mounts[0].line, 4);
    assert_eq!(t.mounts[0].target, "/data");
    assert_eq!(t.swaps[0].priority, 20);
    assert!(!swap::table(&row(0, "auto,noauto,pri=0")).unwrap().swaps[0].automatic);
    assert!(!swap::table(&row(0, "noauto,auto,pri=32767")).unwrap().swaps[0].automatic);
    for options in [
        "defaults",
        "pri=-1",
        "pri=32768",
        "pri=65536",
        "pri=+2",
        "pri=",
        "pri=1,pri=2",
        "pri=1,discard",
        "pri=1,nofail",
        "pri=1,",
    ] {
        assert!(swap::table(&row(0, options)).is_err(), "{options}");
    }
    for text in [
        "/dev/vdb1 /swap swap pri=1 0 0",
        "/dev/vdb1 none swap pri=1 0 2",
        "/work/data none swap pri=1 0 0",
        "/dev/../vdb1 none swap pri=1 0 0",
        "/dev/vdb1 none swap pri=1 0 0 # x",
        "/dev/vdb1 none swap pri=1 0",
        "/dev/vdb1 none swap pri=1 0 0 extra",
    ] {
        assert!(swap::table(text).is_err(), "{text}");
    }
}
#[test]
fn malformed_unsupported_duplicate_and_poisoned_requests_never_publish_a_partial_parse() {
    let f = inventory();
    let mut active = BTreeMap::from([("area1".into(), 15)]);
    let before = active.clone();
    for text in [
        row(0, "pri=1") + &row(2, "pri=1,discard"),
        row(0, "pri=1") + "/dev/vdb1 none swap pri=2 0 0\n",
        row(0, "pri=1").repeat(5),
    ] {
        assert!(swap::activate_all(&f, &mut active, &text).is_err());
        assert_eq!(active, before);
    }
    for source in [
        Source::Device("/dev/vdd1".into()),
        Source::Device("/dev/vde1".into()),
        Source::Device("/dev/unknown".into()),
    ] {
        assert!(swap::activate(&f, &mut active, &source, 10, false).is_err());
        assert_eq!(active, before);
    }
    assert!(
        swap::activate(
            &f,
            &mut active,
            &swap::source(&uuid(0)).unwrap(),
            32768,
            false
        )
        .is_err()
    );
    assert_eq!(active, before);
    assert!(swap::deactivate(&f, &mut active, &swap::source(&uuid(0)).unwrap()).is_err());
    assert_eq!(active, before);
    let mut poison = BTreeMap::from([("unknown".into(), 0)]);
    let old = poison.clone();
    assert!(swap::activate_all(&f, &mut poison, &row(0, "pri=1")).is_err());
    assert_eq!(poison, old);
}
#[test]
fn identities_follow_uuid_when_device_aliases_change_and_inputs_are_bounded() {
    let f = inventory();
    let mut changed = f.clone();
    changed.areas[0].device = "/dev/vdc1".into();
    changed.areas[1].device = "/dev/vdb1".into();
    changed.validate().unwrap();
    assert_eq!(
        changed
            .resolve(&swap::source(&uuid(0)).unwrap())
            .unwrap()
            .id,
        "area0"
    );
    assert_eq!(
        changed
            .resolve(&Source::Device("/dev/vdb1".into()))
            .unwrap()
            .id,
        "area1"
    );
    let mut active = BTreeMap::new();
    swap::activate_all(&changed, &mut active, &row(0, "pri=20")).unwrap();
    assert_eq!(active.keys().collect::<Vec<_>>(), vec!["area0"]);
    let large = "x".repeat(storage::MAX_CONFIG_BYTES + 1);
    assert!(swap::table(&large).is_err());
    let mut bad = f.clone();
    bad.areas.push(f.areas[0].clone());
    assert!(bad.validate().is_err());
    for mutate in [0, 1, 2, 3, 4, 5] {
        let mut bad = f.clone();
        match mutate {
            0 => bad.areas[1].uuid = bad.areas[0].uuid.clone(),
            1 => bad.areas[0].device = "/dev/../personal".into(),
            2 => bad.areas[0].size_bytes = 4095,
            3 => bad.areas[0].size_bytes = (1u64 << 40) + 4096,
            4 => bad.areas[0].id = "".into(),
            _ => bad.areas[0].id = "q".repeat(33),
        }
        assert!(bad.validate().is_err(), "mutation{mutate}");
    }
    let mut v = serde_json::to_value(&f).unwrap();
    v["areas"][0]["device"] = serde_json::json!("q".repeat(65));
    assert!(serde_json::from_value::<swap::Inventory>(v).is_err());
    let mut v = serde_json::to_value(&f).unwrap();
    let extra = v["areas"][0].clone();
    v["areas"].as_array_mut().unwrap().push(extra);
    assert!(serde_json::from_value::<swap::Inventory>(v).is_err());
}
#[test]
fn source_swap_identities_cannot_overlap_existing_mount_inventory() {
    let mounts = storage::Inventory::new(
        BTreeMap::from([(
            "data".into(),
            storage::Volume {
                uuid: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
                filesystem: storage::Filesystem::Ext4,
            },
        )]),
        BTreeMap::from([("/dev/vda1".into(), "data".into())]),
        vec!["/data".into()],
    )
    .unwrap();
    let f = inventory();
    f.validate_against(&mounts).unwrap();
    for n in 0..3 {
        let mut bad = f.clone();
        match n {
            0 => bad.areas[0].id = "data".into(),
            1 => bad.areas[0].device = "/dev/vda1".into(),
            _ => bad.areas[0].uuid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
        };
        assert!(bad.validate_against(&mounts).is_err());
    }
    let before = serde_json::to_value(&mounts).unwrap();
    let old: storage::Inventory = serde_json::from_value(before.clone()).unwrap();
    assert_eq!(serde_json::to_value(old).unwrap(), before);
}
fn mount_inventory() -> storage::Inventory {
    storage::Inventory::new(
        BTreeMap::from([(
            "data".into(),
            storage::Volume {
                uuid: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
                filesystem: storage::Filesystem::Ext4,
            },
        )]),
        BTreeMap::from([("/dev/vda1".into(), "data".into())]),
        vec!["/data".into()],
    )
    .unwrap()
}
fn state_with_swap() -> storage::State {
    storage::State::new(
        mount_inventory().with_swap(inventory()).unwrap(),
        "UUID=aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa /data ext4 defaults 0 2\n".into(),
        BTreeMap::new(),
    )
    .unwrap()
}
#[test]
fn optional_binding_keeps_mount_state_and_saved_and_active_swap_independent() {
    let mut s = state_with_swap();
    s.mount("/data").unwrap();
    let mounts = s.live().clone();
    let original = s.configuration().to_owned();
    s.save_configuration(original + &row(0, "pri=20") + &row(1, "noauto,pri=90"))
        .unwrap();
    assert!(s.active_swap().is_empty());
    assert!(s.validate_configuration().is_empty());
    assert_eq!(s.entries().unwrap().len(), 1);
    assert_eq!(s.swap_entries().unwrap().len(), 2);
    s.swap_on_all().unwrap();
    assert_eq!(s.active_swap()["area0"], 20);
    assert!(!s.active_swap().contains_key("area1"));
    assert_eq!(s.live(), &mounts);
    s.save_configuration(s.configuration().replace("pri=20", "pri=110"))
        .unwrap();
    s.swap_on_all().unwrap();
    assert_eq!(s.active_swap()["area0"], 20);
    s.swap_off(&swap::source(&uuid(0)).unwrap()).unwrap();
    s.swap_on_all().unwrap();
    assert_eq!(s.active_swap()["area0"], 110);
    let bytes = serde_json::to_string(&s).unwrap();
    let restored = storage::State::from_json(&bytes).unwrap();
    assert_eq!(restored, s);
    restored.validate().unwrap();
    let f = s.fresh_session(s.inventory().devices().clone()).unwrap();
    assert_eq!(f.state.live(), &mounts);
    assert_eq!(f.state.active_swap()["area0"], 110);
    assert!(!f.state.active_swap().contains_key("area1"));
    assert_eq!(f.swap_outcomes.len(), 2);
    assert_eq!(s.live(), &mounts);
}
#[test]
fn imported_active_areas_cannot_enable_old_lessons_or_escape_their_source_facts() {
    let s = storage::State::new(mount_inventory(), String::new(), BTreeMap::new()).unwrap();
    let before = serde_json::to_value(&s).unwrap();
    assert!(before.get("active_swap").is_none());
    assert!(before["inventory"].get("swap").is_none());
    let mut raw = before.clone();
    raw["active_swap"] = serde_json::json!({"area0":10});
    assert!(serde_json::from_value::<storage::State>(raw).is_err());
    let mut old = s.clone();
    assert!(old.swap_on_all().is_err());
    assert!(old.swap_on(&swap::source(&uuid(0)).unwrap(), 10).is_err());
    assert_eq!(serde_json::to_value(old).unwrap(), before);
    let mut raw = serde_json::to_value(state_with_swap()).unwrap();
    raw["active_swap"] = serde_json::json!({"area0":32768});
    assert!(serde_json::from_value::<storage::State>(raw).is_err());
    let mut raw = serde_json::to_value(state_with_swap()).unwrap();
    raw["active_swap"] = serde_json::json!({"area2":10});
    assert!(serde_json::from_value::<storage::State>(raw).is_err());
    let enabled = state_with_swap();
    assert!(
        !s.same_inventory(&enabled),
        "Existing authored-goal and progress authority compares complete inventory, including source-only extension"
    );
    let mut changed = inventory();
    changed.areas[0].size_bytes += 4096;
    let foreign = storage::State::new(
        mount_inventory().with_swap(changed).unwrap(),
        String::new(),
        BTreeMap::new(),
    )
    .unwrap();
    assert!(!enabled.same_inventory(&foreign));
}
#[test]
fn all_original_storage_worlds_retain_exact_serialized_shapes_and_mount_only_parser() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/storage-v1.json")).unwrap();
    let mut n = 0;
    for lab in pack["scenarios"].as_array().unwrap() {
        for raw in [
            &lab["initial"]["storage"],
            &lab["storage_teaching"]["fixture"],
        ] {
            assert!(!raw.is_null());
            let state: storage::State = serde_json::from_value(raw.clone()).unwrap();
            assert!(state.inventory().swap().is_none());
            assert!(state.active_swap().is_empty());
            // The original Inventory already canonicalized mountpoints into a
            // BTreeSet. Compare its established serialized shape, not JSON's
            // authored list order; every scalar/object/other array stays exact.
            let mut canonical = raw.clone();
            canonical["inventory"]["mountpoints"]
                .as_array_mut()
                .unwrap()
                .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
            assert_eq!(serde_json::to_value(&state).unwrap(), canonical);
            state.validate().unwrap();
            n += 1;
        }
    }
    assert_eq!(n, 8);
}
#[test]
fn binding_configuration_errors_are_specific_and_partial_bulk_state_is_not_rolled_back() {
    let mut s = state_with_swap();
    let mount = s.configuration().to_owned();
    s.save_configuration(mount.clone() + &row(0, "pri=10") + &row(2, "pri=20"))
        .unwrap();
    let result = s.swap_on_all().unwrap();
    assert!(matches!(result[1], swap::Outcome::Failed { .. }));
    assert_eq!(s.active_swap()["area0"], 10);
    let before = s.clone();
    s.save_configuration(mount.clone() + &row(0, "pri=2") + "/dev/vdb1 none swap pri=3 0 0\n")
        .unwrap();
    assert!(
        s.validate_configuration()
            .iter()
            .any(|e| e.to_string().contains("same supplied area"))
    );
    assert!(s.swap_on_all().is_err());
    assert_eq!(s.active_swap(), before.active_swap());
    s.save_configuration(mount + &row(0, "pri=3,discard"))
        .unwrap();
    assert!(
        s.validate_configuration()[0]
            .to_string()
            .contains("Supported swap options")
    );
    assert!(s.swap_on_all().is_err());
    assert_eq!(s.active_swap(), before.active_swap());
}

#[test]
fn removable_and_ordinary_caller_models_explicitly_refuse_swap_capability() {
    let env = dlms_learning_lab::removable_mount::Environment {
        inventory: mount_inventory().with_swap(inventory()).unwrap(),
        present: std::collections::BTreeSet::from(["data".into()]),
    };
    assert!(
        env.validate()
            .unwrap_err()
            .to_string()
            .contains("do not enable swap")
    );
}

#[test]
fn uuid_case_variants_cannot_bypass_existing_or_optional_identity_guards() {
    // The established mount inventory already requires canonical lowercase.
    assert!(
        storage::Inventory::new(
            BTreeMap::from([(
                "data".into(),
                storage::Volume {
                    uuid: "AAAAAAAA-AAAA-AAAA-AAAA-AAAAAAAAAAAA".into(),
                    filesystem: storage::Filesystem::Ext4
                }
            )]),
            BTreeMap::from([("/dev/vda1".into(), "data".into())]),
            vec!["/data".into()],
        )
        .is_err()
    );
    let lower = mount_inventory();
    let mut facts = inventory();
    facts.areas[0].uuid = "AAAAAAAA-AAAA-AAAA-AAAA-AAAAAAAAAAAA".into();
    assert!(facts.validate().is_err());
    assert!(facts.validate_against(&lower).is_err());
    facts.areas[0].uuid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into();
    facts.validate().unwrap();
    assert!(facts.validate_against(&lower).is_err());
    assert!(lower.with_swap(facts).is_err());
}

#[test]
fn swap_bulk_noauto_presence_is_not_mount_style_last_flag_wins() {
    let facts = inventory();
    for options in [
        "auto,noauto,pri=20",
        "noauto,auto,pri=20",
        "defaults,noauto,auto,pri=20",
        "noauto,defaults,pri=20",
    ] {
        let mut active = BTreeMap::new();
        let text = row(0, options);
        assert!(!swap::table(&text).unwrap().swaps[0].automatic);
        let result = swap::activate_all(&facts, &mut active, &text).unwrap();
        assert!(matches!(&result[0],swap::Outcome::Skipped{id}if id=="area0"));
        assert!(active.is_empty());
        assert!(
            swap::activate(
                &facts,
                &mut active,
                &swap::source(&uuid(0)).unwrap(),
                20,
                false
            )
            .unwrap()
        );
    }
    for options in ["auto,pri=20", "defaults,pri=20", "pri=20"] {
        let mut active = BTreeMap::new();
        let text = row(0, options);
        swap::activate_all(&facts, &mut active, &text).unwrap();
        assert_eq!(active["area0"], 20);
    }
}
