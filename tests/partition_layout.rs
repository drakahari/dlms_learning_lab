use dlms_learning_lab::{
    partition_fs as core,
    partition_fs_workspace::{
        self as work,
        layout::{Plan, Rule},
    },
};
use std::collections::BTreeMap;
const MIB: u64 = 1024 * 1024;
fn world() -> core::State {
    let disk = core::Disk {
        sectors: 65536,
        sector_bytes: 512,
        optimal_alignment: 2048,
        writable: true,
        in_use: false,
        occupied_without_table: false,
        table: None,
        mbr: None,
    };
    let mut state = core::State {
        version: 1,
        mbr_profile: None,
        next_uuid: 1,
        disks: BTreeMap::from([("/dev/vda".into(), disk.clone()), ("/dev/vdb".into(), disk)]),
    };
    for disk in ["/dev/vda", "/dev/vdb"] {
        state
            .apply(core::Action::CreateGpt { disk: disk.into() })
            .unwrap();
    }
    state
}
fn add(state: &mut core::State, disk: &str, name: &str, start: u64, end: u64) {
    state
        .apply(core::Action::AddPartition {
            disk: disk.into(),
            name: name.into(),
            start,
            end,
            type_hint: core::TypeHint::LinuxData,
        })
        .unwrap();
}
fn targets() -> Vec<work::Target> {
    ["root", "logs"]
        .into_iter()
        .map(|name| work::Target {
            name: name.into(),
            eligible_disks: vec!["/dev/vda".into(), "/dev/vdb".into()],
            type_hint: core::TypeHint::LinuxData,
            minimum_bytes: 512,
            maximum_bytes: 20 * MIB,
            aligned_start: true,
            content: work::DesiredContent::Blank,
        })
        .collect()
}
fn adjacent(bytes: u64) -> Plan {
    Plan {
        version: 1,
        rules: vec![Rule::AdjacentFree {
            target: "logs".into(),
            minimum_bytes: bytes,
        }],
    }
}
#[test]
fn contiguous_growth_is_not_total_free_space_and_respects_both_inclusive_edges() {
    let mut state = world();
    add(&mut state, "/dev/vda", "logs", 2048, 8191);
    add(&mut state, "/dev/vda", "root", 14336, 20479);
    let mut policy = adjacent(3 * MIB);
    assert!(policy.checks(&state, &targets())[0].0);
    policy.rules[0] = Rule::AdjacentFree {
        target: "logs".into(),
        minimum_bytes: 3 * MIB + 512,
    };
    let checks = policy.checks(&state, &targets());
    assert!(!checks[0].0 && checks[0].1.contains("3145728"));
    assert!(state.disks["/dev/vda"].sectors * 512 > 20 * MIB); // Plenty of other space is not adjacent.
}
#[test]
fn gpt_backup_metadata_never_becomes_future_growth_capacity() {
    let mut state = world();
    let last = state.disks["/dev/vda"].sectors - 34;
    add(&mut state, "/dev/vda", "logs", 2048, last - 8);
    assert!(adjacent(4096).checks(&state, &targets())[0].0);
    assert!(!adjacent(4608).checks(&state, &targets())[0].0);
    let mut state = world();
    let last = state.disks["/dev/vda"].sectors - 34;
    add(&mut state, "/dev/vda", "logs", 2048, last);
    assert!(!adjacent(512).checks(&state, &targets())[0].0);
}
#[test]
fn separation_uses_real_current_allocations_and_accepts_either_disk_assignment() {
    let policy = Plan {
        version: 1,
        rules: vec![Rule::SeparateDisks {
            first: "root".into(),
            second: "logs".into(),
        }],
    };
    for (root, logs) in [("/dev/vda", "/dev/vdb"), ("/dev/vdb", "/dev/vda")] {
        let mut state = world();
        add(&mut state, root, "root", 2048, 8191);
        add(&mut state, logs, "logs", 8192, 14335);
        assert!(policy.checks(&state, &targets())[0].0);
    }
    let mut state = world();
    add(&mut state, "/dev/vda", "root", 2048, 8191);
    add(&mut state, "/dev/vda", "logs", 8192, 14335);
    assert!(!policy.checks(&state, &targets())[0].0);
}
#[test]
fn missing_ambiguous_or_malformed_layouts_never_receive_credit() {
    let policy = adjacent(MIB);
    let mut state = world();
    assert!(!policy.checks(&state, &targets())[0].0);
    add(&mut state, "/dev/vda", "logs", 2048, 8191);
    add(&mut state, "/dev/vdb", "logs", 2048, 8191);
    assert!(!policy.checks(&state, &targets())[0].0);
    state.disks.get_mut("/dev/vda").unwrap().sectors = 1;
    assert!(!policy.checks(&state, &targets())[0].0);
}
#[test]
fn invalid_unknown_duplicate_and_oversized_source_policies_fail_before_use() {
    for bytes in [0, 513, core::MAX_SECTORS * core::SECTOR_BYTES + 512] {
        assert!(adjacent(bytes).validate(&targets()).is_err());
    }
    let mut bad = adjacent(MIB);
    bad.version = 2;
    assert!(bad.validate(&targets()).is_err());
    bad = adjacent(MIB);
    bad.rules.push(Rule::AdjacentFree {
        target: "logs".into(),
        minimum_bytes: 2 * MIB,
    });
    assert!(bad.validate(&targets()).is_err());
    bad = Plan {
        version: 1,
        rules: vec![Rule::SeparateDisks {
            first: "root".into(),
            second: "missing".into(),
        }],
    };
    assert!(bad.validate(&targets()).is_err());
    bad = Plan {
        version: 1,
        rules: vec![
            Rule::SeparateDisks {
                first: "root".into(),
                second: "logs".into(),
            },
            Rule::SeparateDisks {
                first: "logs".into(),
                second: "root".into(),
            },
        ],
    };
    assert!(bad.validate(&targets()).is_err());
    let mut value = serde_json::to_value(adjacent(MIB)).unwrap();
    value["rules"][0]["target"] = "x".repeat(37).into();
    assert!(serde_json::from_value::<Plan>(value).is_err());
    let mut value = serde_json::to_value(adjacent(MIB)).unwrap();
    value["rules"] = serde_json::Value::Array(vec![value["rules"][0].clone(); 5]);
    assert!(serde_json::from_value::<Plan>(value).is_err());
}
#[test]
fn old_named_plans_retain_their_exact_wire_value_and_new_rules_are_opt_in() {
    let raw = serde_json::json!({"targets":targets()});
    let old: work::Plan = serde_json::from_value(raw.clone()).unwrap();
    assert_eq!(serde_json::to_value(old).unwrap(), raw);
    let mut expanded = raw.clone();
    expanded["layout"] = serde_json::to_value(adjacent(MIB)).unwrap();
    let next: work::Plan = serde_json::from_value(expanded.clone()).unwrap();
    assert_eq!(serde_json::to_value(next).unwrap(), expanded);
}
