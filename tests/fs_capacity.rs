//! Unintegrated pure foundation tests. No filesystem/process/reference calls.
use dlms_learning_lab::fs_capacity as capacity;
use capacity::{
    Error, MAX_FILES, MAX_LOGICAL_BYTES, MAX_POOL, MAX_STATE_BYTES, Pressure, Profile, State,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn profile() -> Profile {
    Profile {
        id: "reports".into(),
        block_bytes: 4096,
        total_blocks: 16,
        fixed_used_blocks: 2,
        root_reserved_blocks: 3,
        total_inodes: 16,
        fixed_used_inodes: 2,
        retained: BTreeMap::from([("/keep".into(), 1)]),
    }
}
fn fixture() -> State {
    State::new(
        profile(),
        BTreeMap::from([
            ("/keep".into(), 1),
            ("/cache".into(), 4097),
            ("/empty".into(), 0),
        ]),
    )
    .unwrap()
}
fn value(s: &State) -> Value {
    serde_json::to_value(s).unwrap()
}
fn from_value(v: Value) -> State {
    State::from_json(&v.to_string()).unwrap()
}
#[test]
fn independent_accounting_separates_metadata_logical_data_free_and_available() {
    let s = fixture();
    let u = s.usage().unwrap();
    assert_eq!(u.block_bytes, 4096);
    assert_eq!(
        (
            u.total_blocks,
            u.used_blocks,
            u.free_blocks,
            u.available_blocks
        ),
        (16, 5, 11, 8)
    );
    assert_eq!((u.file_logical_bytes, u.file_data_blocks), (4098, 3));
    assert_eq!((u.total_inodes, u.used_inodes, u.free_inodes), (16, 5, 11));
    assert_eq!(
        (u.total_bytes, u.free_bytes, u.available_bytes),
        (65536, 45056, 32768)
    );
    assert_eq!((u.used_percent, u.inode_percent), (Some(39), Some(32)));
    assert_eq!(u.pressure(), Pressure::Neither);
    assert_eq!(s.profile().unwrap(), &profile());
}
#[test]
fn dense_rounding_and_empty_inodes_hold_at_each_supported_block_boundary() {
    for block_bytes in [1024, 2048, 4096] {
        for n in [0, 1, block_bytes - 1, block_bytes, block_bytes + 1] {
            let mut p = profile();
            p.block_bytes = block_bytes;
            p.retained.clear();
            let s = State::new(p, BTreeMap::from([("/f".into(), n)])).unwrap();
            let expected = if n == 0 {
                0
            } else if n <= block_bytes {
                1
            } else {
                2
            };
            let u = s.usage().unwrap();
            assert_eq!(u.used_blocks, 2 + expected);
            assert_eq!(u.used_inodes, 3);
        }
    }
}
#[test]
fn empty_files_exhaust_inodes_while_blocks_remain_and_failure_is_atomic() {
    let mut p = profile();
    p.retained.clear();
    p.total_inodes = 4;
    let mut s = State::new(p, BTreeMap::from([("/a".into(), 0), ("/b".into(), 0)])).unwrap();
    assert_eq!(s.usage().unwrap().pressure(), Pressure::Inodes);
    let before = s.clone();
    assert_eq!(s.put_file("/c", 0), Err(Error::NoInodes));
    assert_eq!(s, before);
    s.put_file("/a", 1).unwrap();
    assert_eq!(
        (
            s.usage().unwrap().free_inodes,
            s.usage().unwrap().used_blocks
        ),
        (0, 3)
    );
}
#[test]
fn reserved_free_blocks_are_unavailable_but_do_not_count_as_used() {
    let mut p = profile();
    p.retained.clear();
    let mut s = State::new(p, BTreeMap::from([("/f".into(), 11 * 4096)])).unwrap();
    let u = s.usage().unwrap();
    assert_eq!(
        (
            u.used_blocks,
            u.free_blocks,
            u.available_blocks,
            u.used_percent
        ),
        (13, 3, 0, Some(100))
    );
    assert_eq!(u.pressure(), Pressure::OrdinaryBlocks);
    let before = s.clone();
    assert_eq!(s.put_file("/f", 11 * 4096 + 1), Err(Error::NoBlocks));
    assert_eq!(s, before);
    s.put_file("/zero", 0).unwrap(); // data exhaustion is different from inode exhaustion
    assert_eq!(s.usage().unwrap().free_blocks, 3);
}
#[test]
fn both_resources_and_zero_percentage_denominator_are_reported_explicitly() {
    let p = Profile {
        total_blocks: 4,
        fixed_used_blocks: 0,
        root_reserved_blocks: 4,
        total_inodes: 1,
        fixed_used_inodes: 0,
        retained: BTreeMap::new(),
        ..profile()
    };
    let mut s = State::new(p, BTreeMap::new()).unwrap();
    assert_eq!(
        (
            s.usage().unwrap().used_percent,
            s.usage().unwrap().inode_percent
        ),
        (None, Some(0))
    );
    s.put_file("/empty", 0).unwrap();
    assert_eq!(s.usage().unwrap().pressure(), Pressure::Both);
}
#[test]
fn supplied_reserved_occupancy_can_shrink_but_ordinary_growth_cannot_spend_reserve() {
    let mut p = profile();
    p.retained.clear();
    let mut s = State::new(p, BTreeMap::from([("/f".into(), 13 * 4096)])).unwrap();
    assert_eq!(
        (
            s.usage().unwrap().free_blocks,
            s.usage().unwrap().available_blocks
        ),
        (1, 0)
    );
    let before = s.clone();
    assert_eq!(s.put_file("/f", 13 * 4096 + 1), Err(Error::NoBlocks));
    assert_eq!(s, before);
    s.put_file("/f", 9 * 4096).unwrap();
    assert_eq!(s.usage().unwrap().available_blocks, 2);
    s.put_file("/f", 11 * 4096).unwrap();
    assert_eq!(s.usage().unwrap().available_blocks, 0);
}
#[test]
fn current_pressure_returns_after_post_cleanup_growth_and_no_stale_success_is_stored() {
    let mut s = fixture();
    let before = s.clone();
    let plan = s.preview_remove(&["/cache", "/empty"]).unwrap();
    assert_eq!(s, before); // preview is read-only
    assert_eq!(
        (plan.projected().free_blocks, plan.projected().free_inodes),
        (13, 13)
    );
    s.apply_remove(&plan).unwrap();
    assert_eq!((s.files().unwrap().len(), s.generation().unwrap()), (1, 1));
    s.put_file("/cache", 10 * 4096).unwrap();
    assert_eq!(s.usage().unwrap().available_blocks, 0);
    assert_eq!(s.usage().unwrap().pressure(), Pressure::OrdinaryBlocks);
    assert_eq!(s.files().unwrap().get("/keep"), Some(&1));
}
#[test]
fn protected_file_size_and_presence_cannot_be_changed_or_previewed_away() {
    let mut s = fixture();
    let before = s.clone();
    assert_eq!(s.put_file("/keep", 1), Err(Error::Protected));
    assert_eq!(s.put_file("/keep", 2), Err(Error::Protected));
    assert!(matches!(
        s.preview_remove(&["/cache", "/keep"]),
        Err(Error::Protected)
    ));
    assert_eq!(s, before);
    let mut raw = value(&s);
    raw["files"].as_object_mut().unwrap().remove("/keep");
    assert!(State::from_json(&raw.to_string()).is_err());
    let mut raw = value(&s);
    raw["files"]["/keep"] = json!(2);
    assert!(State::from_json(&raw.to_string()).is_err());
}
#[test]
fn duplicate_unknown_bad_path_and_late_failure_plans_never_partially_remove() {
    let s = fixture();
    let before = s.clone();
    for (names, error) in [
        (vec!["/cache", "/cache"], Error::Invalid),
        (vec!["/cache", "/missing"], Error::Unknown),
        (vec!["/cache", "/../keep"], Error::Invalid),
    ] {
        assert!(matches!(s.preview_remove(&names),Err(e) if e==error));
        assert_eq!(s, before);
    }
    assert!(matches!(s.preview_remove(&[]), Err(Error::Limit)));
    assert!(matches!(
        s.preview_remove(&vec!["/cache"; MAX_FILES + 1]),
        Err(Error::Limit)
    ));
}
#[test]
fn whole_state_preview_refuses_unrelated_changes_and_write_restore_aba() {
    let mut s = fixture();
    let plan = s.preview_remove(&["/cache"]).unwrap();
    s.put_file("/unrelated", 0).unwrap();
    let current = s.clone();
    assert_eq!(s.apply_remove(&plan), Err(Error::Stale));
    assert_eq!(s, current);
    let mut s = fixture();
    let plan = s.preview_remove(&["/cache"]).unwrap();
    s.put_file("/empty", 1).unwrap();
    s.put_file("/empty", 0).unwrap();
    assert_eq!(s.files().unwrap(), fixture().files().unwrap());
    let current = s.clone();
    assert_eq!(s.apply_remove(&plan), Err(Error::Stale));
    assert_eq!(s, current);
}
#[test]
fn no_op_does_not_change_generation_and_applied_plan_cannot_be_replayed() {
    let mut s = fixture();
    let before = s.clone();
    let plan = s.preview_remove(&["/cache"]).unwrap();
    s.put_file("/empty", 0).unwrap();
    assert_eq!(s, before);
    s.apply_remove(&plan).unwrap();
    let after = s.clone();
    assert_eq!(s.apply_remove(&plan), Err(Error::Stale));
    assert_eq!(s, after);
}
#[test]
fn generation_exhaustion_refuses_changes_and_previews_without_mutation() {
    let mut raw = value(&fixture());
    raw["generation"] = json!(u64::MAX);
    let mut s = from_value(raw);
    let before = s.clone();
    assert_eq!(s.put_file("/empty", 1), Err(Error::Limit));
    assert_eq!(s, before);
    assert!(matches!(s.preview_remove(&["/empty"]), Err(Error::Limit)));
    assert_eq!(s, before);
    s.put_file("/empty", 0).unwrap();
    assert_eq!(s, before);
}
#[test]
fn environment_is_the_complete_valid_supplied_profile_not_current_file_mutations() {
    let original = fixture();
    let mut changed = original.clone();
    changed.put_file("/empty", 10).unwrap();
    assert!(changed.same_environment(&original));
    assert!(original.same_environment(&changed));
    let mut p = profile();
    p.root_reserved_blocks = 1;
    let other = State::new(p, original.files().unwrap().clone()).unwrap();
    assert!(!other.same_environment(&original));
    assert!(!original.same_environment(&other));
    let mut p = profile();
    p.block_bytes = 8192;
    assert_eq!(
        State::new(p, original.files().unwrap().clone()),
        Err(Error::Invalid)
    );
}
#[test]
fn validated_compact_and_direct_serde_roundtrips_reject_poisoned_fields() {
    let s = fixture();
    let raw = s.to_json().unwrap();
    assert_eq!(State::from_json(&raw).unwrap(), s);
    assert_eq!(serde_json::from_str::<State>(&raw).unwrap(), s);
    for (key, bad) in [
        ("version", json!(2)),
        ("generation", json!(-1)),
        ("other", json!(0)),
    ] {
        let mut v = value(&s);
        v[key] = bad;
        assert!(serde_json::from_value::<State>(v).is_err());
    }
    for (key, bad) in [
        ("total_blocks", 0),
        ("total_inodes", 1),
        ("block_bytes", 512),
        ("fixed_used_blocks", 17),
        ("root_reserved_blocks", 17),
    ] {
        let mut v = value(&s);
        v["profile"][key] = json!(bad);
        assert!(State::from_json(&v.to_string()).is_err());
    }
}
#[test]
fn direct_decoder_rejects_duplicate_fields_keys_and_excess_count_before_values() {
    let raw = fixture().to_json().unwrap();
    let duplicate = raw.replacen("\"version\":1", "\"version\":1,\"version\":1", 1);
    assert!(serde_json::from_str::<State>(&duplicate).is_err());
    let duplicate = raw.replacen("\"/empty\":0", "\"/empty\":0,\"/empty\":0", 1);
    let e = serde_json::from_str::<State>(&duplicate)
        .unwrap_err()
        .to_string();
    assert!(e.contains("duplicate before value"));
    let mut raw = value(&fixture());
    raw["profile"]["retained"] = json!({});
    raw["files"] = Value::Object(
        (0..MAX_FILES)
            .map(|n| (format!("/f{n}"), json!(0)))
            .collect(),
    );
    raw["profile"]["total_inodes"] = json!(MAX_FILES + 2);
    let raw = raw
        .to_string()
        .replacen("\"files\":{", "\"files\":{\"/extra\":0,", 1);
    let e = serde_json::from_str::<State>(&raw).unwrap_err().to_string();
    assert!(e.contains("bound/duplicate before value"));
}
#[test]
fn unsafe_keys_oversized_bytes_and_raw_cap_refuse_atomically() {
    let mut s = fixture();
    let before = s.clone();
    for name in [
        "", "relative", "/", "/.", "/..", "/a/b", "/a/", "//a", "/a\n", "/é", "/a;rm",
    ] {
        assert_eq!(s.put_file(name, 0), Err(Error::Invalid));
        assert_eq!(s, before);
    }
    assert_eq!(
        s.put_file("/large", MAX_LOGICAL_BYTES + 1),
        Err(Error::Limit)
    );
    assert_eq!(s, before);
    assert_eq!(
        State::from_json(&" ".repeat(MAX_STATE_BYTES + 1)),
        Err(Error::Limit)
    );
    let mut raw = value(&s);
    raw["files"]["/large"] = json!(u64::MAX);
    assert!(serde_json::from_value::<State>(raw).is_err());
}
#[test]
fn bounded_profile_arithmetic_refuses_overcommit_and_file_count_without_overflow() {
    let mut p = profile();
    p.total_blocks = MAX_POOL + 1;
    assert_eq!(p.validate(), Err(Error::Invalid));
    let mut p = profile();
    p.fixed_used_inodes = 17;
    assert_eq!(p.validate(), Err(Error::Invalid));
    let mut p = profile();
    p.retained.clear();
    assert_eq!(
        State::new(p.clone(), BTreeMap::from([("/f".into(), 15 * 4096)])),
        Err(Error::Invalid)
    );
    let files = (0..MAX_FILES + 1).map(|n| (format!("/f{n}"), 0)).collect();
    assert_eq!(State::new(p, files), Err(Error::Invalid));
}
#[test]
fn all_maximal_name_maps_fit_the_existing_raw_cap_without_large_data_allocation() {
    let files: BTreeMap<String, u64> = (0..MAX_FILES).map(|n| (format!("/{n:047}"), 0)).collect();
    let p = Profile {
        id: "p".repeat(47),
        total_blocks: MAX_POOL,
        fixed_used_blocks: MAX_POOL - 1,
        root_reserved_blocks: MAX_POOL,
        total_inodes: MAX_POOL,
        fixed_used_inodes: MAX_POOL - MAX_FILES as u64,
        retained: files.clone(),
        ..profile()
    };
    let s = State::new(p, files).unwrap();
    let raw = s.to_json().unwrap();
    assert!(raw.len() < 20_000);
    assert!(raw.len() < MAX_STATE_BYTES);
    assert_eq!(State::from_json(&raw).unwrap(), s);
    assert_eq!(
        (
            s.usage().unwrap().free_inodes,
            s.usage().unwrap().pressure()
        ),
        (0, Pressure::Both)
    );
}

#[test]
fn aggregate_logical_limit_and_oversized_codec_key_refuse_without_allocating_file_contents() {
    let mut p = profile();
    p.total_blocks = MAX_POOL;
    p.fixed_used_blocks = 0;
    p.retained.clear();
    let mut s = State::new(p, BTreeMap::from([("/large".into(), MAX_LOGICAL_BYTES)])).unwrap();
    let before = s.clone();
    assert_eq!(s.put_file("/another", 1), Err(Error::Limit));
    assert_eq!(s, before);
    assert!(s.to_json().unwrap().len() < 1024);
    let mut v = value(&s);
    v["profile"]["unexpected"] = json!(1);
    assert!(serde_json::from_value::<State>(v).is_err());
    let raw = s
        .to_json()
        .unwrap()
        .replace("/large", &format!("/{}", "x".repeat(48)));
    let e = serde_json::from_str::<State>(&raw).unwrap_err().to_string();
    assert!(e.contains("text bound before copy"));
}
