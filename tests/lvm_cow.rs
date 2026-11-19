//! Sparse COW prerequisite proof only; no authored lesson or host-LVM acceptance.
use dlms_learning_lab::lvm::{
    MIB,
    cow::{Bank, Slot},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
fn bank() -> Bank {
    Bank::new(
        128 * 1024,
        BTreeMap::from([
            (
                "first".into(),
                (
                    Slot {
                        offset: 100,
                        length: 8,
                    },
                    "record-A".into(),
                ),
            ),
            (
                "shared".into(),
                (
                    Slot {
                        offset: 200,
                        length: 8,
                    },
                    "record-B".into(),
                ),
            ),
            (
                "distant".into(),
                (
                    Slot {
                        offset: 8192,
                        length: 8,
                    },
                    "record-C".into(),
                ),
            ),
        ]),
    )
    .unwrap()
}
#[test]
fn snapshots_start_without_origin_copies_and_first_writes_preserve_a_whole_shared_chunk() {
    let mut b = bank();
    b.create("before", MIB, 4096).unwrap();
    assert_eq!(b.status("before").unwrap().copied_chunks, 0);
    assert_eq!(b.read("before", "first").unwrap(), "record-A");
    assert!(b.write("first", "edited-A").unwrap().invalidated.is_empty());
    assert_eq!(b.status("before").unwrap().copied_chunks, 1);
    let value = serde_json::to_value(&b).unwrap();
    let images = value["snapshots"]["before"]["images"].as_object().unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(
        images["0"]["fragments"].as_object().unwrap().len(),
        2,
        "Shared chunk includes both records, not a per-file backup"
    );
    assert!(images["0"]["fragments"].get("distant").is_none());
    b.write("shared", "edited-B").unwrap();
    b.write("first", "latest-A").unwrap();
    assert_eq!(
        b.status("before").unwrap().copied_chunks,
        1,
        "Repeated/shared writes charge once"
    );
    assert_eq!(b.read("before", "first").unwrap(), "record-A");
    assert_eq!(b.read("before", "shared").unwrap(), "record-B");
    assert_eq!(b.read("before", "distant").unwrap(), "record-C");
    assert_eq!(b.current("first").unwrap(), "latest-A");
}
#[test]
fn different_snapshot_ages_retain_distinct_bytes_and_chunk_sizes() {
    let mut b = bank();
    b.create("early", MIB, 4096).unwrap();
    b.write("first", "edited-A").unwrap();
    b.create("later", MIB, 8192).unwrap();
    assert_eq!(b.status("later").unwrap().copied_chunks, 0);
    b.write("first", "latest-A").unwrap();
    b.write("distant", "edited-C").unwrap();
    assert_eq!(b.read("early", "first").unwrap(), "record-A");
    assert_eq!(b.read("later", "first").unwrap(), "edited-A");
    assert_eq!(b.read("early", "distant").unwrap(), "record-C");
    assert_eq!(b.read("later", "distant").unwrap(), "record-C");
}
#[test]
fn a_record_crossing_a_chunk_boundary_charges_two_and_reads_back_exactly() {
    let mut b = Bank::new(
        16384,
        BTreeMap::from([
            (
                "crossing".into(),
                (
                    Slot {
                        offset: 4092,
                        length: 8,
                    },
                    "abcdefgh".into(),
                ),
            ),
            (
                "neighbor".into(),
                (
                    Slot {
                        offset: 4112,
                        length: 8,
                    },
                    "untouchd".into(),
                ),
            ),
        ]),
    )
    .unwrap();
    b.create("saved", 3 * 4096, 4096).unwrap();
    b.write("crossing", "ABCDEFGH").unwrap();
    assert_eq!(b.status("saved").unwrap().copied_chunks, 2);
    assert_eq!(b.read("saved", "crossing").unwrap(), "abcdefgh");
    assert_eq!(b.read("saved", "neighbor").unwrap(), "untouchd");
    b.write("crossing", "12345678").unwrap();
    assert_eq!(b.status("saved").unwrap().copied_chunks, 2);
}
#[test]
fn exhaustion_invalidates_snapshot_but_preserves_origin_writes_and_cannot_be_revived() {
    let mut b = bank();
    b.create("small", 8192, 4096).unwrap();
    assert_eq!(b.status("small").unwrap().usable_chunks, 1);
    b.write("first", "edited-A").unwrap();
    let result = b.write("distant", "edited-C").unwrap();
    assert_eq!(result.invalidated, vec!["small"]);
    assert_eq!(b.current("distant").unwrap(), "edited-C");
    assert!(!b.status("small").unwrap().valid);
    assert!(
        b.read("small", "first")
            .unwrap_err()
            .message
            .contains("invalid")
    );
    let before = b.clone();
    assert!(b.extend("small", MIB).is_err());
    assert_eq!(b, before);
    assert!(b.write("first", "latest-A").unwrap().invalidated.is_empty());
    assert!(!b.status("small").unwrap().valid);
    b.remove("small").unwrap();
    b.create("small", MIB, 4096).unwrap();
    assert_eq!(
        b.read("small", "first").unwrap(),
        "latest-A",
        "Recreation is a new time, not recovery of lost before-images"
    );
}
#[test]
fn adding_space_before_exhaustion_preserves_existing_exceptions_and_refusals_are_atomic() {
    let mut b = bank();
    b.create("save", 8192, 4096).unwrap();
    b.write("first", "edited-A").unwrap();
    let images = serde_json::to_value(&b).unwrap()["snapshots"]["save"]["images"].clone();
    b.extend("save", 3 * 4096).unwrap();
    b.write("distant", "edited-C").unwrap();
    assert!(b.status("save").unwrap().valid);
    assert_eq!(b.status("save").unwrap().copied_chunks, 2);
    assert_eq!(
        serde_json::to_value(&b).unwrap()["snapshots"]["save"]["images"]["0"],
        images["0"]
    );
    for (bytes, chunk) in [
        (4096, 4096),
        (MIB, 3000),
        (MIB, 0),
        (MIB, 1024 * 1024),
        (MIB + 1, 4096),
    ] {
        let before = b.clone();
        assert!(b.create("wrong", bytes, chunk).is_err());
        assert_eq!(b, before);
    }
    for bytes in [8192, 3 * 4096] {
        let before = b.clone();
        assert!(b.extend("save", bytes).is_err());
        assert_eq!(b, before);
    }
    let before = b.clone();
    assert!(b.create("save", MIB, 4096).is_err());
    assert_eq!(b, before);
}
#[test]
fn deterministic_sparse_outcomes_match_an_independent_byte_array_reference() {
    let records = (0..12)
        .map(|i| {
            let text = format!("r{i:02}-original");
            let slot = Slot {
                offset: i * 8192 + 4092,
                length: text.len() as u32,
            };
            (format!("row{i}"), (slot, text))
        })
        .collect::<BTreeMap<_, _>>();
    let mut b = Bank::new(128 * 1024, records.clone()).unwrap();
    let mut origin = vec![0u8; b.byte_len() as usize];
    for (slot, text) in records.values() {
        origin[slot.offset as usize..(slot.offset + u64::from(slot.length)) as usize]
            .copy_from_slice(text.as_bytes());
    }
    // Full arrays exist ONLY in this independent bounded test oracle.
    let mut references: BTreeMap<String, (Vec<u8>, u64, BTreeSet<u64>, bool)> = BTreeMap::new();
    b.create("early", 128 * 4096, 4096).unwrap();
    references.insert(
        "early".into(),
        (origin.clone(), 4096, BTreeSet::new(), true),
    );
    for step in 0..72 {
        if step == 9 {
            b.create("late", 2 * 8192, 8192).unwrap();
            references.insert("late".into(), (origin.clone(), 8192, BTreeSet::new(), true));
        }
        let id = format!("row{}", (step * 7 + 3) % 12);
        let (slot, old) = &records[&id];
        let data = std::iter::repeat_n(char::from(b'A' + (step % 26) as u8), old.len())
            .collect::<String>();
        b.write(&id, &data).unwrap();
        origin[slot.offset as usize..(slot.offset + u64::from(slot.length)) as usize]
            .copy_from_slice(data.as_bytes());
        for (name, (saved, chunk, touched, valid)) in &mut references {
            if *valid {
                touched.extend(
                    slot.offset / *chunk..=(slot.offset + u64::from(slot.length) - 1) / *chunk,
                );
                let usable = if name == "early" { 127 } else { 1 };
                if touched.len() > usable {
                    *valid = false;
                }
            }
            assert_eq!(b.status(name).unwrap().valid, *valid);
            for (record, (slot, _)) in &records {
                assert_eq!(
                    b.current(record).unwrap().as_bytes(),
                    &origin[slot.offset as usize..(slot.offset + u64::from(slot.length)) as usize]
                );
                if *valid {
                    assert_eq!(
                        b.read(name, record).unwrap().as_bytes(),
                        &saved
                            [slot.offset as usize..(slot.offset + u64::from(slot.length)) as usize]
                    );
                } else {
                    assert!(b.read(name, record).is_err());
                }
            }
        }
    }
}
#[test]
fn roundtrip_and_malformed_sparse_images_fail_before_admission() {
    let mut b = bank();
    b.create("save", MIB, 4096).unwrap();
    b.write("first", "edited-A").unwrap();
    let raw = serde_json::to_vec(&b).unwrap();
    assert_eq!(Bank::from_json(&raw).unwrap(), b);
    let value: Value = serde_json::from_slice(&raw).unwrap();
    let reject = |q: Value| assert!(Bank::from_json(&serde_json::to_vec(&q).unwrap()).is_err());
    let mut q = value.clone();
    q["layout"]["shared"]["offset"] = json!(104);
    reject(q);
    let mut q = value.clone();
    q["layout"]["first"]["offset"] = json!(u64::MAX);
    reject(q);
    let mut q = value.clone();
    q["snapshots"]["save"]["images"]["0"]["fragments"]
        .as_object_mut()
        .unwrap()
        .remove("shared");
    reject(q);
    let mut q = value.clone();
    let v = q["snapshots"]["save"]["images"]
        .as_object_mut()
        .unwrap()
        .remove("0")
        .unwrap();
    q["snapshots"]["save"]["images"]["00"] = v;
    reject(q);
    let mut q = value.clone();
    q["snapshots"]["save"]["images"]["0"]["fragments"]["first"]["offset"] = json!(0);
    reject(q);
    let mut q = value.clone();
    q["current"]["first"]["contents"] = json!("éééé");
    reject(q);
    let mut q = value.clone();
    q["snapshots"]["save"]["chunk_bytes"] = json!(1);
    reject(q);
    let mut q = value;
    q["foreign"] = json!(true);
    reject(q);
    assert!(Bank::from_json(&vec![b' '; 1024 * 1024 + 1]).is_err());
}
#[test]
fn fixed_record_limits_unknown_names_and_snapshot_inventory_refuse_without_side_effects() {
    let mut b = bank();
    let before = b.clone();
    for (name, data) in [
        ("first", "short"),
        ("first", "\x1b1234567"),
        ("first", "éééé"),
        ("../escape", "12345678"),
        ("missing", "12345678"),
    ] {
        assert!(b.write(name, data).is_err());
        assert_eq!(b, before);
    }
    for id in ["../escape", "/dev/sda", "snapshot", "bad/name"] {
        assert!(b.create(id, MIB, 4096).is_err());
        assert_eq!(b, before);
    }
    for id in ["one", "two", "three", "four"] {
        b.create(id, MIB, 4096).unwrap();
    }
    let before = b.clone();
    assert!(b.create("five", MIB, 4096).is_err());
    assert_eq!(b, before);
    assert!(Bank::new(512, BTreeMap::new()).is_err());
    assert!(
        Bank::new(
            512,
            BTreeMap::from([(
                "x".into(),
                (
                    Slot {
                        offset: 0,
                        length: 0
                    },
                    String::new()
                )
            )])
        )
        .is_err()
    );
}

#[test]
fn snapshot_identities_are_never_reused_and_counter_bounds_refuse_atomically() {
    let mut b = bank();
    b.create("same", MIB, 4096).unwrap();
    let original = b.clone();
    b.remove("same").unwrap();
    b.create("same", MIB, 4096).unwrap();
    assert_eq!(
        b.read("same", "first").unwrap(),
        original.read("same", "first").unwrap()
    );
    assert!(b.continues(&original));
    assert!(!b.keeps_snapshot(&original, "same", false));
    let mut raw = serde_json::to_value(&original).unwrap();
    raw["next_snapshot_id"] = json!(0);
    assert!(Bank::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    let mut raw = serde_json::to_value(&original).unwrap();
    raw["snapshots"]["duplicate"] = raw["snapshots"]["same"].clone();
    assert!(Bank::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    let mut raw = serde_json::to_value(&original).unwrap();
    raw["next_snapshot_id"] = json!(u64::MAX);
    let mut full = Bank::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    let before = full.clone();
    assert!(full.create("new", MIB, 4096).is_err());
    assert_eq!(full, before);
}
