//! Snapshot storage prerequisite; no admitted native lesson or actual LVM run.
use dlms_learning_lab::{
    lvm::{
        self, Extent, MIB, State,
        cow::{Bank, Slot},
        snapshot,
    },
    lvm_workspace as work,
};
use std::collections::BTreeMap;
#[path = "support/lvm_snapshot_fixture.rs"]
mod fixture;
use fixture::{bank, initial, plain, same_neighbors};
#[test]
fn new_profile_reserves_distinct_actual_extents_and_retains_original_allocations() {
    let mut s = initial();
    let before = s.clone();
    let free = s.free_extents("work").unwrap();
    s.execute("lvcreate -s -L 1M -n saved -c 128K work/report")
        .unwrap();
    assert_eq!(s.free_extents("work").unwrap(), free - 1);
    assert_eq!(s.lv_bytes("work/saved").unwrap(), MIB);
    assert_eq!(bank(&s).byte_len(), 2 * MIB);
    assert_eq!(bank(&s).status("saved").unwrap().copied_chunks, 0);
    assert_eq!(bank(&s).status("saved").unwrap().usable_chunks, 7);
    let r = s.execute("lvs").unwrap();
    assert!(r.contains("SIMULATOR COW facts"));
    assert!(r.contains("2097152 1048576 0 7 Valid"));
    same_neighbors(&s, &before);
    s.validate().unwrap();
}
#[test]
fn alternate_options_paths_units_and_default_chunks_produce_valid_equivalent_state() {
    let mut a = initial();
    let mut b = a.clone();
    a.execute("lvcreate -s -L 1M -n saved -c 128K work/report")
        .unwrap();
    b.execute("lvcreate --name saved --chunksize 128 --size 1m --snapshot /dev/work/report")
        .unwrap();
    assert_eq!(a, b);
    let mut c = initial();
    c.execute("lvcreate --snapshot --size 1 --name saved work/report")
        .unwrap();
    assert_eq!(bank(&c).status("saved").unwrap().chunk_bytes, 4096);
    let mut d = initial();
    d.execute("lvcreate -s -L 1048577 -n saved work/report")
        .unwrap_err(); // MiB default exceeds supplied free extents, never bytes.
    assert_eq!(d, initial());
}
#[test]
fn real_origin_writes_restore_snapshot_records_and_keep_two_ages_independent() {
    let mut s = initial();
    s.execute("lvcreate -s -L 1M -n early -c 128 work/report")
        .unwrap();
    snapshot::write_record(&mut s, "/dev/work/report", "row0", "edited-0").unwrap();
    s.execute("lvcreate -s -L 1M -n later -c 128 work/report")
        .unwrap();
    snapshot::write_record(&mut s, "work/report", "row0", "latest-0").unwrap();
    assert_eq!(
        snapshot::read_record(&s, "work/report", Some("early"), "row0").unwrap(),
        "record-0"
    );
    assert_eq!(
        snapshot::read_record(&s, "work/report", Some("later"), "row0").unwrap(),
        "edited-0"
    );
    snapshot::restore_record(&mut s, "work/report", "early", "row0").unwrap();
    assert_eq!(
        snapshot::read_record(&s, "work/report", None, "row0").unwrap(),
        "record-0"
    );
    assert_eq!(
        snapshot::read_record(&s, "work/report", Some("later"), "row0").unwrap(),
        "edited-0"
    );
    assert_eq!(bank(&s).status("early").unwrap().copied_chunks, 1);
}
#[test]
fn exhausted_store_does_not_stop_origin_or_leak_extents_on_failed_extension() {
    let mut s = initial();
    s.execute("lvcreate -s -L 1M -n small -c 128 work/report")
        .unwrap();
    for i in 0..7 {
        assert!(
            snapshot::write_record(
                &mut s,
                "work/report",
                &format!("row{i}"),
                &format!("edited-{i}")
            )
            .unwrap()
            .invalidated
            .is_empty()
        );
    }
    let result = snapshot::write_record(&mut s, "work/report", "row7", "edited-7").unwrap();
    assert_eq!(result.invalidated, vec!["small"]);
    assert_eq!(
        snapshot::read_record(&s, "work/report", None, "row7").unwrap(),
        "edited-7"
    );
    assert!(!bank(&s).status("small").unwrap().valid);
    assert!(
        s.execute("lvs")
            .unwrap()
            .contains("Invalid · data unavailable")
    );
    let before = s.clone();
    assert!(s.execute("lvextend -L +1M work/small").is_err());
    assert_eq!(s, before);
    assert!(snapshot::restore_record(&mut s, "work/report", "small", "row0").is_err());
    assert_eq!(s, before);
}
#[test]
fn extend_before_exhaustion_uses_free_physical_space_and_removal_frees_only_cow() {
    let mut s = initial();
    let original = s.clone();
    s.execute("lvcreate -s -L 1M -n saved -c 128K work/report")
        .unwrap();
    for i in 0..7 {
        snapshot::write_record(
            &mut s,
            "work/report",
            &format!("row{i}"),
            &format!("edited-{i}"),
        )
        .unwrap();
    }
    let mut alternate = s.clone();
    s.execute("lvextend -L +1M work/saved").unwrap();
    alternate
        .execute("lvextend --size 2 /dev/work/saved")
        .unwrap();
    assert_eq!(s, alternate);
    snapshot::write_record(&mut s, "work/report", "row7", "edited-7").unwrap();
    assert!(bank(&s).status("saved").unwrap().valid);
    assert_eq!(
        s.free_extents("work").unwrap(),
        original.free_extents("work").unwrap() - 2
    );
    let expected = bank(&s).current("row0").unwrap().to_owned();
    s.execute("lvremove --yes /dev/work/saved").unwrap();
    assert_eq!(
        s.free_extents("work").unwrap(),
        original.free_extents("work").unwrap()
    );
    assert_eq!(
        snapshot::read_record(&s, "work/report", None, "row0").unwrap(),
        expected
    );
    assert!(bank(&s).status("saved").is_err());
    same_neighbors(&s, &original);
}
#[test]
fn unsupported_destructive_origin_unknown_and_no_space_inputs_are_atomic() {
    let mut s = initial();
    let before = s.clone();
    for input in [
        "lvremove -y work/report",
        "lvremove -y work/reference",
        "lvcreate -s -L 10M -n huge work/report",
        "lvcreate -s -L 1M -n report work/report",
        "lvcreate -s -L 1M -n saved work/reference",
        "lvcreate -s -L 1M -n saved -c 3 work/report",
        "lvcreate -s -L 1M -n saved -c 128M work/report",
        "lvcreate -s -s -L 1M -n saved work/report",
        "lvcreate -s -L 1M -L 2M -n saved work/report",
        "lvcreate -s -L 1M -n ../escape work/report",
        "lvcreate -s -L 1M -n saved /dev/sda",
        "lvconvert --merge work/report",
        "pvs; touch /tmp/forbidden",
        "lvextend -L +1M work/report",
        "lvcreate -s -L1M -n saved work/report",
    ] {
        assert!(s.execute(input).is_err(), "{input}");
        assert_eq!(s, before, "{input}");
    }
    for (name, value) in [
        ("missing", "record-0"),
        ("row0", "short"),
        ("row0", "éééé"),
        ("../../escape", "record-0"),
    ] {
        assert!(snapshot::write_record(&mut s, "work/report", name, value).is_err());
        assert_eq!(s, before);
    }
}
#[test]
fn physical_metadata_corruption_mixing_and_wire_injection_fail_without_changing_old_schema() {
    let old = plain();
    old.validate().unwrap();
    let json = serde_json::to_value(&old).unwrap();
    assert!(json.get("snapshots").is_none());
    assert_eq!(
        State::from_json(&serde_json::to_vec(&old).unwrap()).unwrap(),
        old
    );
    let mut old_command = old.clone();
    assert!(
        old_command
            .execute("lvcreate -s -L 1M -n saved work/report")
            .is_err()
    );
    assert_eq!(old_command, old);
    let mut s = initial();
    s.execute("lvcreate -s -L 1M -n saved work/report").unwrap();
    assert_eq!(
        State::from_json(&serde_json::to_vec(&s).unwrap()).unwrap(),
        s
    );
    let mut wrong = s.clone();
    wrong
        .groups
        .get_mut("work")
        .unwrap()
        .volumes
        .get_mut("saved")
        .unwrap()
        .extents
        .push(Extent {
            pv: "/dev/store".into(),
            index: 4,
        });
    assert!(wrong.validate().is_err());
    let mut wrong = s.clone();
    wrong.provisioning = Some(lvm::provision::Profile::Linear);
    assert!(wrong.validate().is_err());
    let mut wrong = s.clone();
    wrong
        .snapshots
        .as_mut()
        .unwrap()
        .origins
        .insert("/dev/work/report".into(), bank(&s).clone());
    assert!(wrong.validate().is_err());
    let mut wrong = s.clone();
    wrong
        .groups
        .get_mut("work")
        .unwrap()
        .volumes
        .remove("saved");
    assert!(wrong.validate().is_err());
    let mut wrong = s;
    wrong.snapshots.as_mut().unwrap().origins.insert(
        "work/saved".into(),
        Bank::new(
            MIB,
            BTreeMap::from([(
                "x".into(),
                (
                    Slot {
                        offset: 0,
                        length: 1,
                    },
                    "x".into(),
                ),
            )]),
        )
        .unwrap(),
    );
    assert!(wrong.validate().is_err());
}
#[test]
fn current_native_authoring_gate_and_old_goal_context_refuse_joint_snapshot_injection() {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/lvm-growth-v1.json")).unwrap();
    let mut d: work::Definition =
        serde_json::from_value(raw["scenarios"][0]["goal"]["definition"].clone()).unwrap();
    let old = d.initial.clone();
    let s = initial();
    assert!(!work::same_problem(&old, &s));
    assert!(!work::same_problem(&s, &s));
    d.initial = s;
    assert!(
        d.shape().unwrap_err().contains("not admitted"),
        "Native teaching/typed outcomes must exist before authoring admission"
    );
}

#[test]
fn largest_supported_chunks_share_records_and_honor_exact_store_capacity() {
    let mut s = initial();
    s.execute("lvcreate --snapshot --size 1M --name large --chunksize 512K work/report")
        .unwrap();
    assert_eq!(bank(&s).status("large").unwrap().chunk_bytes, 512 * 1024);
    snapshot::write_record(&mut s, "work/report", "row3", "edited-3").unwrap();
    snapshot::write_record(&mut s, "work/report", "row0", "edited-0").unwrap();
    assert_eq!(bank(&s).status("large").unwrap().copied_chunks, 1);
    for i in 0..4 {
        assert_eq!(
            snapshot::read_record(&s, "work/report", Some("large"), &format!("row{i}")).unwrap(),
            format!("record-{i}")
        );
    }
    assert_eq!(
        snapshot::write_record(&mut s, "work/report", "row4", "edited-4")
            .unwrap()
            .invalidated,
        vec!["large"]
    );
    assert!(!bank(&s).status("large").unwrap().valid);
}
