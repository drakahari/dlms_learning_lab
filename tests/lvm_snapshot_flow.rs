//! Typed stimuli stay separate from grading and permit meaningful repairs.
use dlms_learning_lab::{
    lvm::{self, MIB},
    lvm_workspace::{snapshot as goal, snapshot_flow::*},
};
#[path = "support/lvm_snapshot_fixture.rs"]
mod fixture;
use fixture::{bank, initial, plain, same_neighbors};
fn command(s: &str) -> Operation {
    Operation::Command { text: s.into() }
}
fn context() -> Context {
    Context {
        plan: goal::Plan {
            origins: vec![goal::Origin {
                volume: "work/report".into(),
                records: vec![goal::Record {
                    record: "row0".into(),
                    expected: goal::Value::Text {
                        contents: "edited-0".into(),
                    },
                }],
            }],
            snapshots: vec![goal::Snapshot {
                origin: "work/report".into(),
                name: "saved".into(),
                minimum_bytes: MIB,
                maximum_bytes: 2 * MIB,
                chunk_bytes: Some(128 * 1024),
                records: (0..8)
                    .map(|i| goal::Record {
                        record: format!("row{i}"),
                        expected: goal::Value::Original,
                    })
                    .collect(),
            }],
            retire: vec![],
            reserves: vec![goal::Reserve {
                group: "work".into(),
                minimum_bytes: 5 * MIB,
            }],
        },
        updates: vec![
            Update {
                label: "New report".into(),
                explanation: "The simulated application changes one record after the saved point."
                    .into(),
                writes: vec![Change {
                    origin: "work/report".into(),
                    record: "row0".into(),
                    contents: "edited-0".into(),
                }],
            },
            Update {
                label: "Another update".into(),
                explanation:
                    "This writes the remaining distinct chunks and can exhaust a small COW store."
                        .into(),
                writes: (1..8)
                    .map(|i| Change {
                        origin: "work/report".into(),
                        record: format!("row{i}"),
                        contents: format!("edited-{i}"),
                    })
                    .collect(),
            },
        ],
    }
}
fn reference() -> Vec<Operation> {
    vec![
        command("lvcreate -s -L 1M -n saved -c 128 work/report"),
        Operation::ApplyUpdate { index: 0 },
    ]
}
fn passes(c: &Context, source: &lvm::State, s: &lvm::State) -> bool {
    c.checks(source, s).iter().all(|r| r.0)
}
fn solve(c: &Context, source: &lvm::State) -> lvm::State {
    let mut s = source.clone();
    for op in reference() {
        op.apply(c, source, &mut s).unwrap();
    }
    s
}
fn practice() -> Practice {
    Practice {
        context: context(),
        worked: reference()
            .into_iter()
            .map(|operation| Step {
                operation,
                explanation: "Run the operation, then inspect the current and saved records."
                    .into(),
            })
            .collect(),
        experiment: command("lvcreate -s -L 1M -n extra -c 128 work/report"),
        repair: vec![command("lvremove -y work/extra")],
    }
}
#[test]
fn reference_and_alternatives_grade_real_data_without_mutating_source() {
    let source = initial();
    let saved = source.clone();
    let c = context();
    Task {
        context: c.clone(),
        reference: reference(),
    }
    .validate(&source)
    .unwrap();
    let a = solve(&c, &source);
    assert!(passes(&c, &source, &a));
    same_neighbors(&a, &source);
    let mut b = source.clone();
    command("lvcreate --chunksize 128K --snapshot --name saved --size 2m /dev/work/report")
        .apply(&c, &source, &mut b)
        .unwrap();
    Operation::ApplyUpdate { index: 0 }
        .apply(&c, &source, &mut b)
        .unwrap();
    assert!(passes(&c, &source, &b));
    assert_ne!(a, b);
    assert_eq!(source, saved);
    assert_ne!(source, plain());
}
#[test]
fn extra_view_fails_grading_but_does_not_lock_out_explicit_repair() {
    let source = initial();
    let p = practice();
    p.validate(&source).unwrap();
    let mut s = solve(&p.context, &source);
    p.experiment.apply(&p.context, &source, &mut s).unwrap();
    assert!(goal::same_context(&source, &s));
    assert!(!passes(&p.context, &source, &s));
    p.repair[0].apply(&p.context, &source, &mut s).unwrap();
    assert!(passes(&p.context, &source, &s));
}
#[test]
fn invalid_old_view_requires_reset_or_new_point_not_fake_extension_recovery() {
    let source = initial();
    let c = context();
    let mut s = solve(&c, &source);
    let output = Operation::ApplyUpdate { index: 1 }
        .apply(&c, &source, &mut s)
        .unwrap();
    assert!(output.contains("invalid") && output.contains("origin write succeeded"));
    assert!(!passes(&c, &source, &s));
    assert!(!bank(&s).status("saved").unwrap().valid);
    let saved = s.clone();
    assert!(
        command("lvextend -L +1M work/saved")
            .apply(&c, &source, &mut s)
            .is_err()
    );
    assert_eq!(s, saved);
    command("lvremove -y work/saved")
        .apply(&c, &source, &mut s)
        .unwrap();
    command("lvcreate -s -L 2M -n saved -c 128 work/report")
        .apply(&c, &source, &mut s)
        .unwrap();
    assert!(bank(&s).status("saved").unwrap().valid);
    assert!(!passes(&c, &source, &s), "new age is not old data");
    let mut p = practice();
    p.experiment = Operation::ApplyUpdate { index: 1 };
    p.repair.clear();
    p.validate(&source).unwrap();
    assert_eq!(initial(), source);
}
#[test]
fn explicitly_simulated_record_copy_uses_actual_saved_bytes_and_can_repair_current_data() {
    let source = initial();
    let mut c = context();
    c.plan.origins[0].records[0].expected = goal::Value::Original;
    let mut s = source.clone();
    command("lvcreate -s -L 2M -n saved -c 128 work/report")
        .apply(&c, &source, &mut s)
        .unwrap();
    Operation::ApplyUpdate { index: 0 }
        .apply(&c, &source, &mut s)
        .unwrap();
    assert!(!passes(&c, &source, &s));
    let copy = Operation::CopyRecord {
        origin: "work/report".into(),
        snapshot: "saved".into(),
        record: "row0".into(),
    };
    assert!(copy.description(&c).starts_with("Simulator record copy"));
    copy.apply(&c, &source, &mut s).unwrap();
    assert_eq!(bank(&s).current("row0").unwrap(), "record-0");
    assert!(passes(&c, &source, &s));
    Operation::ApplyUpdate { index: 0 }
        .apply(&c, &source, &mut s)
        .unwrap();
    assert!(!passes(&c, &source, &s));
}
#[test]
fn assignment_and_separate_practice_drafts_state_and_credit_are_not_aliased() {
    let source = initial();
    let c = context();
    let assignment = source.clone();
    let mut example = source.clone();
    for op in reference() {
        op.apply(&c, &source, &mut example).unwrap();
    }
    assert!(passes(&c, &source, &example));
    assert!(!passes(&c, &source, &assignment));
    assert_eq!(assignment, source);
    let bytes = serde_json::to_vec(&example).unwrap();
    let restored = lvm::State::from_json(&bytes).unwrap();
    assert_eq!(restored, example);
    assert!(passes(&c, &source, &restored));
    assert!(!passes(&c, &source, &source));
}
#[test]
fn source_geometry_and_unrelated_data_damage_are_rejected_without_blocking_normal_mistakes() {
    let source = initial();
    let c = context();
    let mut s = solve(&c, &source);
    lvm::snapshot::write_record(&mut s, "work/report", "row2", "edited-2").unwrap();
    assert!(goal::same_context(&source, &s));
    assert!(!passes(&c, &source, &s));
    let mut foreign = s.clone();
    foreign
        .groups
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
        .contents = "Foreign change\n".into();
    let saved = foreign.clone();
    assert!(!goal::same_context(&source, &foreign));
    assert!(command("lvs").apply(&c, &source, &mut foreign).is_err());
    assert_eq!(foreign, saved);
    let mut value = serde_json::to_value(&source).unwrap();
    value["snapshots"]["origins"]["work/report"]["layout"]["row0"]["offset"] = 101.into();
    let mut foreign = lvm::State::from_json(&serde_json::to_vec(&value).unwrap()).unwrap();
    let saved = foreign.clone();
    assert!(
        Operation::ApplyUpdate { index: 0 }
            .apply(&c, &source, &mut foreign)
            .is_err()
    );
    assert_eq!(foreign, saved);
}
#[test]
fn malformed_update_unsupported_host_commands_and_unknown_records_are_atomic() {
    let source = initial();
    let c = context();
    for op in [
        command("sh -c lvs"),
        command("lvcreate -s -L 1M -n saved work/report; id"),
        command("lvconvert --merge work/saved"),
        Operation::ApplyUpdate { index: 4 },
        Operation::CopyRecord {
            origin: "work/report".into(),
            snapshot: "saved".into(),
            record: "unknown".into(),
        },
    ] {
        let mut s = source.clone();
        assert!(op.apply(&c, &source, &mut s).is_err());
        assert_eq!(s, source);
    }
    let mut malformed = c.clone();
    malformed.updates[0].writes.push(Change {
        origin: "work/report".into(),
        record: "row1".into(),
        contents: "wrong length".into(),
    });
    let mut s = source.clone();
    assert!(
        Operation::ApplyUpdate { index: 0 }
            .apply(&malformed, &source, &mut s)
            .is_err()
    );
    assert_eq!(s, source);
    let mut value = serde_json::to_value(&c).unwrap();
    value["updates"][0]["shell"] = "real host command".into();
    assert!(serde_json::from_value::<Context>(value).is_err());
}
#[test]
fn authoring_rejects_wrong_age_noop_exploration_fake_repairs_and_unbounded_metadata() {
    let source = initial();
    let mut t = Task {
        context: context(),
        reference: reference(),
    };
    t.reference.swap(0, 1);
    assert!(t.validate(&source).is_err());
    let mut p = practice();
    p.experiment = command("lvs");
    assert!(p.validate(&source).is_err());
    let mut p = practice();
    p.repair = vec![command("lvs")];
    assert!(p.validate(&source).is_err());
    let mut p = practice();
    p.worked[0].explanation = " ".into();
    assert!(p.validate(&source).is_err());
    let mut c = context();
    let duplicate = c.updates[0].writes[0].clone();
    c.updates[0].writes.push(duplicate);
    assert!(c.shape(&source).is_err());
    let mut c = context();
    c.updates = vec![c.updates[0].clone(); 5];
    assert!(c.shape(&source).is_err());
    let mut value = serde_json::to_value(context()).unwrap();
    value["updates"] = serde_json::json!(vec![value["updates"][0].clone(); 5]);
    assert!(serde_json::from_value::<Context>(value).is_err());
}
