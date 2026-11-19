//! Typed outcomes only. Normal authoring remains closed pending native teaching.
use dlms_learning_lab::{
    lvm::{self, MIB, snapshot},
    lvm_workspace::snapshot as goal,
};
#[path = "support/lvm_snapshot_fixture.rs"]
mod fixture;
use fixture::{bank, initial, plain, same_neighbors};
fn original(name: &str) -> goal::Record {
    goal::Record {
        record: name.into(),
        expected: goal::Value::Original,
    }
}
fn text(name: &str, value: &str) -> goal::Record {
    goal::Record {
        record: name.into(),
        expected: goal::Value::Text {
            contents: value.into(),
        },
    }
}
fn plan() -> goal::Plan {
    goal::Plan {
        origins: vec![goal::Origin {
            volume: "work/report".into(),
            records: vec![text("row0", "edited-0")],
        }],
        snapshots: vec![goal::Snapshot {
            origin: "work/report".into(),
            name: "saved".into(),
            minimum_bytes: MIB,
            maximum_bytes: 2 * MIB,
            chunk_bytes: Some(128 * 1024),
            records: (0..8).map(|i| original(&format!("row{i}"))).collect(),
        }],
        retire: vec![],
        reserves: vec![goal::Reserve {
            group: "work".into(),
            minimum_bytes: 5 * MIB,
        }],
    }
}
fn all(before: &lvm::State, s: &lvm::State, p: &goal::Plan) -> bool {
    goal::outcomes(before, s, p).iter().all(|r| r.0)
}
fn solved(before: &lvm::State) -> lvm::State {
    let mut s = before.clone();
    s.execute("lvcreate -s -L 1M -n saved -c 128K work/report")
        .unwrap();
    snapshot::write_record(&mut s, "work/report", "row0", "edited-0").unwrap();
    s
}
#[test]
fn alternate_allocations_and_option_forms_solve_actual_data_not_transcripts() {
    let before = initial();
    let p = plan();
    p.validate(&before).unwrap();
    assert!(!all(&before, &before, &p));
    let a = solved(&before);
    assert!(
        all(&before, &a, &p),
        "{:?}",
        goal::outcomes(&before, &a, &p)
    );
    same_neighbors(&a, &before);
    let mut b = before.clone();
    b.execute("lvcreate --chunksize 128 --name saved --snapshot --size 2m /dev/work/report")
        .unwrap();
    snapshot::write_record(&mut b, "work/report", "row0", "edited-0").unwrap();
    assert!(all(&before, &b, &p));
    assert_ne!(
        a.groups["work"].volumes["saved"].extents,
        b.groups["work"].volumes["saved"].extents
    );
}
#[test]
fn partial_wrong_age_wrong_chunks_and_later_breakage_have_specific_current_failures() {
    let before = initial();
    let p = plan();
    let mut partial = before.clone();
    partial
        .execute("lvcreate -s -L 1M -n saved -c 128K work/report")
        .unwrap();
    let rows = goal::outcomes(&before, &partial, &p);
    assert!(!rows[0].0 && rows[1].0);
    assert!(rows[0].1.contains("row0"));
    let mut late = before.clone();
    snapshot::write_record(&mut late, "work/report", "row0", "edited-0").unwrap();
    late.execute("lvcreate -s -L 1M -n saved -c 128K work/report")
        .unwrap();
    let rows = goal::outcomes(&before, &late, &p);
    assert!(rows[0].0 && !rows[1].0);
    assert!(rows[1].1.contains("point-in-time"));
    let mut wrong = before.clone();
    wrong
        .execute("lvcreate -s -L 1M -n saved -c 4K work/report")
        .unwrap();
    snapshot::write_record(&mut wrong, "work/report", "row0", "edited-0").unwrap();
    assert!(!all(&before, &wrong, &p));
    assert!(
        goal::outcomes(&before, &wrong, &p)[1]
            .1
            .contains("chunk size")
    );
    let mut s = solved(&before);
    assert!(all(&before, &s, &p));
    snapshot::write_record(&mut s, "work/report", "row0", "broken-0").unwrap();
    assert!(!all(&before, &s, &p));
    let mut exhausted = solved(&before);
    for i in 1..8 {
        snapshot::write_record(
            &mut exhausted,
            "work/report",
            &format!("row{i}"),
            &format!("edited-{i}"),
        )
        .unwrap();
    }
    let rows = goal::outcomes(&before, &exhausted, &p);
    assert!(rows[1].1.contains("invalid"));
    assert!(!all(&before, &exhausted, &p));
}
#[test]
fn unexpected_records_views_neighbor_damage_and_changed_layout_cannot_retain_success() {
    let before = initial();
    let p = plan();
    let mut s = solved(&before);
    snapshot::write_record(&mut s, "work/report", "row2", "edited-2").unwrap();
    let rows = goal::outcomes(&before, &s, &p);
    assert!(rows[0].0 && rows[1].0 && !rows.last().unwrap().0);
    let mut s = solved(&before);
    s.execute("lvcreate -s -L 1M -n extra -c 128 work/report")
        .unwrap();
    assert!(!goal::same_problem(&before, &s, &p));
    let mut s = solved(&before);
    s.groups
        .get_mut("work")
        .unwrap()
        .volumes
        .get_mut("reference")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .get_mut("/keep.txt")
        .unwrap()
        .contents = "Damaged neighbor\n".into();
    s.validate().unwrap();
    assert!(!all(&before, &s, &p));
    let mut value = serde_json::to_value(&before).unwrap();
    value["snapshots"]["origins"]["work/report"]["layout"]["row0"]["offset"] =
        serde_json::json!(101);
    let mut s = lvm::State::from_json(&serde_json::to_vec(&value).unwrap()).unwrap();
    s.execute("lvcreate -s -L 1M -n saved -c 128 work/report")
        .unwrap();
    snapshot::write_record(&mut s, "work/report", "row0", "edited-0").unwrap();
    assert!(!goal::same_problem(&before, &s, &p));
}
#[test]
fn supplied_view_identity_cannot_be_replaced_with_a_new_matching_name_and_bytes() {
    let mut before = initial();
    before
        .execute("lvcreate -s -L 1M -n original -c 128 work/report")
        .unwrap();
    let mut p = plan();
    p.snapshots[0].name = "next".into();
    let mut s = before.clone();
    s.execute("lvcreate -s -L 1M -n next -c 128 work/report")
        .unwrap();
    snapshot::write_record(&mut s, "work/report", "row0", "edited-0").unwrap();
    assert!(all(&before, &s, &p));
    let bank_before = bank(&before).clone();
    assert!(bank(&s).keeps_snapshot(&bank_before, "original", false));
    s.execute("lvremove -y work/original").unwrap();
    snapshot::write_record(&mut s, "work/report", "row0", "record-0").unwrap();
    s.execute("lvcreate -s -L 1M -n original -c 128 work/report")
        .unwrap();
    snapshot::write_record(&mut s, "work/report", "row0", "edited-0").unwrap();
    assert_eq!(
        snapshot::read_record(&s, "work/report", Some("original"), "row0").unwrap(),
        "record-0"
    );
    assert!(!bank(&s).keeps_snapshot(&bank_before, "original", false));
    assert!(
        !all(&before, &s, &p),
        "Identical names/data do not invent original snapshot identity"
    );
}
#[test]
fn retiring_a_supplied_invalid_view_requires_its_absence_and_keeps_origin_data() {
    let mut before = initial();
    before
        .execute("lvcreate -s -L 1M -n old -c 128 work/report")
        .unwrap();
    for i in 0..8 {
        snapshot::write_record(
            &mut before,
            "work/report",
            &format!("row{i}"),
            &format!("edited-{i}"),
        )
        .unwrap();
    }
    assert!(!bank(&before).status("old").unwrap().valid);
    let mut p = plan();
    p.origins[0].records = vec![text("row0", "latest-0")];
    p.retire.push(goal::Retire {
        volume: "work/old".into(),
    });
    p.snapshots[0].name = "fresh".into();
    p.validate(&before).unwrap();
    let mut s = before.clone();
    s.execute("lvcreate -s -L 1M -n fresh -c 128 work/report")
        .unwrap();
    snapshot::write_record(&mut s, "work/report", "row0", "latest-0").unwrap();
    assert!(!all(&before, &s, &p));
    s.execute("lvremove -y work/old").unwrap();
    assert!(
        all(&before, &s, &p),
        "{:?}",
        goal::outcomes(&before, &s, &p)
    );
    same_neighbors(&s, &before);
}
#[test]
fn invalid_plans_foreign_contexts_and_malformed_data_are_not_credited() {
    let source = initial();
    let p = plan();
    assert!(p.validate(&plain()).is_err());
    assert!(!all(&source, &plain(), &p));
    let mut q = p.clone();
    q.snapshots.clear();
    assert!(q.validate(&source).is_err());
    let mut q = p.clone();
    q.snapshots.push(q.snapshots[0].clone());
    assert!(q.validate(&source).is_err());
    let mut q = p.clone();
    q.snapshots[0].name = "report".into();
    assert!(q.validate(&source).is_err());
    let mut q = p.clone();
    q.snapshots[0].origin = "other/report".into();
    assert!(q.validate(&source).is_err());
    let mut q = p.clone();
    q.origins[0].records[0].expected = goal::Value::Text {
        contents: "short".into(),
    };
    assert!(q.validate(&source).is_err());
    let mut q = p.clone();
    q.snapshots[0].chunk_bytes = Some(0);
    assert!(q.validate(&source).is_err());
    let mut q = p.clone();
    q.retire.push(goal::Retire {
        volume: "work/report".into(),
    });
    assert!(q.validate(&source).is_err());
    let mut q = p.clone();
    q.reserves[0].group = "unknown".into();
    assert!(q.validate(&source).is_err());
    let mut raw = serde_json::to_value(&p).unwrap();
    raw["snapshots"][0]["unreviewed"] = serde_json::json!(true);
    assert!(serde_json::from_value::<goal::Plan>(raw).is_err());
}
