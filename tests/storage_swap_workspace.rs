//! Behavior-based swap outcomes over the existing saved/live storage workspace.
use dlms_learning_lab::{
    storage::{self, swap as core},
    storage_workspace::{self as work, swap},
};
use std::collections::BTreeMap;
const DATA: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const AUDIT: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
const MAIN: &str = "11111111-1111-4111-8111-111111111111";
const RESERVE: &str = "22222222-2222-4222-8222-222222222222";
fn state() -> storage::State {
    let inventory = storage::Inventory::new(
        BTreeMap::from([
            (
                "data".into(),
                storage::Volume {
                    uuid: DATA.into(),
                    filesystem: storage::Filesystem::Ext4,
                },
            ),
            (
                "audit".into(),
                storage::Volume {
                    uuid: AUDIT.into(),
                    filesystem: storage::Filesystem::Ext4,
                },
            ),
        ]),
        BTreeMap::from([
            ("/dev/vda1".into(), "data".into()),
            ("/dev/vdb1".into(), "audit".into()),
        ]),
        vec!["/data".into(), "/audit".into()],
    )
    .unwrap()
    .with_swap(core::Inventory {
        areas: vec![
            core::Area {
                id: "primary".into(),
                device: "/dev/vdc1".into(),
                uuid: MAIN.into(),
                size_bytes: 1 << 30,
                prepared: true,
                available: true,
            },
            core::Area {
                id: "reserve".into(),
                device: "/dev/vdd1".into(),
                uuid: RESERVE.into(),
                size_bytes: 1 << 29,
                prepared: true,
                available: true,
            },
        ],
    })
    .unwrap();
    let mut s = storage::State::new(inventory, policy(false, 20), BTreeMap::new())
        .unwrap()
        .with_active_swap(BTreeMap::from([
            ("primary".into(), 10),
            ("reserve".into(), 5),
        ]))
        .unwrap();
    s.mount("/data").unwrap();
    s.mount("/audit").unwrap();
    s
}
fn policy(auto: bool, pri: u16) -> String {
    format!(
        "UUID={DATA} /data ext4 defaults 0 2\nUUID={AUDIT} /audit ext4 ro 0 2\nUUID={MAIN} none swap {},pri={pri} 0 0\nUUID={RESERVE} none swap noauto,pri=5 0 0\n",
        if auto { "auto" } else { "noauto" }
    )
}
fn requirements() -> Vec<work::Requirement> {
    let wrap = |r| work::Requirement::Swap { requirement: r };
    vec![
        work::Requirement::Configured {
            target: "/data".into(),
            volume: "data".into(),
            read_only: false,
            automatic: true,
            dump: 0,
            pass: 2,
        },
        work::Requirement::Configured {
            target: "/audit".into(),
            volume: "audit".into(),
            read_only: true,
            automatic: true,
            dump: 0,
            pass: 2,
        },
        work::Requirement::Mounted {
            target: "/data".into(),
            volume: "data".into(),
            read_only: false,
        },
        work::Requirement::Mounted {
            target: "/audit".into(),
            volume: "audit".into(),
            read_only: true,
        },
        work::Requirement::FreshSession {
            devices: BTreeMap::from([
                ("/dev/vdb1".into(), "data".into()),
                ("/dev/vda1".into(), "audit".into()),
            ]),
            expected: BTreeMap::from([
                (
                    "/data".into(),
                    Some(storage::Mount {
                        volume: "data".into(),
                        read_only: false,
                    }),
                ),
                (
                    "/audit".into(),
                    Some(storage::Mount {
                        volume: "audit".into(),
                        read_only: true,
                    }),
                ),
            ]),
        },
        wrap(swap::Requirement::Configured {
            area: "primary".into(),
            automatic: true,
            priority: 80,
        }),
        wrap(swap::Requirement::Active {
            area: "primary".into(),
            priority: 80,
        }),
        wrap(swap::Requirement::Configured {
            area: "reserve".into(),
            automatic: false,
            priority: 5,
        }),
        wrap(swap::Requirement::Active {
            area: "reserve".into(),
            priority: 5,
        }),
        wrap(swap::Requirement::Fresh {
            devices: BTreeMap::from([
                ("/dev/vdd1".into(), "primary".into()),
                ("/dev/vdc1".into(), "reserve".into()),
            ]),
            expected: BTreeMap::from([("primary".into(), Some(80)), ("reserve".into(), None)]),
        }),
    ]
}
fn cmd(s: &mut storage::State, text: &str) {
    work::command(s, text).unwrap_or_else(|e| panic!("{text}: {e}"));
}
#[test]
fn state_grading_accepts_equivalent_orders_and_clears_success_when_either_policy_breaks() {
    let rs = requirements();
    let source = state();
    assert!(work::valid_requirements(&rs, source.inventory()));
    assert!(work::checks(&source, &rs).iter().any(|r| !r.0));
    for variant in 0..2 {
        let mut s = source.clone();
        let protected = s.live().clone();
        if variant == 0 {
            s.save_configuration(policy(true, 80)).unwrap();
            cmd(&mut s, "swapoff /dev/vdc1");
            cmd(&mut s, "swapon -a");
        } else {
            cmd(&mut s, &format!("swapoff -U {MAIN}"));
            cmd(&mut s, &format!("swapon -p 80 -U {MAIN}"));
            s.save_configuration(policy(true, 80).replace("auto,pri=80", "pri=80,defaults"))
                .unwrap();
        }
        assert!(
            work::checks(&s, &rs).iter().all(|r| r.0),
            "{:?}",
            work::checks(&s, &rs)
        );
        assert_eq!(s.live(), &protected);
        assert_eq!(s.active_swap()["reserve"], 5);
        cmd(&mut s, "swapoff /dev/vdc1");
        let bad = work::checks(&s, &rs);
        assert!(bad.iter().any(|r| !r.0 && r.1.contains("not active")));
        assert!(bad.iter().filter(|r| r.0).count() >= 8);
        cmd(&mut s, "swapon -a");
        assert!(work::checks(&s, &rs).iter().all(|r| r.0));
        s.save_configuration(policy(false, 80)).unwrap();
        let bad = work::checks(&s, &rs);
        assert!(
            bad.iter()
                .any(|r| !r.0 && r.1.contains("automatic activation"))
        );
        assert!(
            bad.iter()
                .any(|r| !r.0 && r.1.contains("Fresh swap-policy"))
        );
    }
}
#[test]
fn saved_only_live_only_wrong_uuid_and_blank_policy_produce_useful_partial_feedback() {
    let rs = requirements();
    let mut s = state();
    s.save_configuration(policy(true, 80)).unwrap();
    cmd(&mut s, "swapon -a");
    let rows = work::checks(&s, &rs);
    let failed = rows.iter().filter(|r| !r.0).collect::<Vec<_>>();
    assert_eq!(failed.len(), 1);
    assert!(failed[0].1.contains("priority is 10"));
    assert!(failed[0].1.contains("skips an already-active"));
    assert!(
        work::command(&mut s, "swapon -p 80 /dev/vdc1")
            .unwrap_err()
            .contains("already active")
    );
    assert_eq!(s.active_swap()["primary"], 10);
    cmd(&mut s, "swapoff /dev/vdc1");
    cmd(&mut s, "swapon -p 80 /dev/vdc1");
    assert!(work::checks(&s, &rs).iter().all(|r| r.0));
    s.save_configuration(policy(true, 80).replace(MAIN, "99999999-9999-4999-8999-999999999999"))
        .unwrap();
    let rows = work::checks(&s, &rs);
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("No supplied swap area"))
    );
    assert!(rows.iter().any(|r| r.0 && r.1.contains("Active primary")));
    s.save_configuration(String::new()).unwrap();
    assert!(work::checks(&s, &rs).iter().filter(|r| !r.0).count() >= 5);
    let mut only_live = state();
    cmd(&mut only_live, "swapoff /dev/vdc1");
    cmd(&mut only_live, "swapon -p 80 /dev/vdc1");
    let rows = work::checks(&only_live, &rs);
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("Saved primary")));
    assert!(rows.iter().any(|r| r.0 && r.1.contains("Active primary")));
}
#[test]
fn fresh_policy_uses_independent_alias_permutation_not_current_active_rows() {
    let mut s = state();
    s.save_configuration(policy(true, 80)).unwrap();
    let before = s.clone();
    let aliases = BTreeMap::from([
        ("/dev/vdd1".into(), "primary".into()),
        ("/dev/vdc1".into(), "reserve".into()),
    ]);
    let fresh = swap::fresh(&s, &aliases).unwrap();
    assert_eq!(fresh.active, BTreeMap::from([("primary".into(), 80)]));
    assert_eq!(s, before);
    assert_eq!(fresh.inventory.areas[0].device, "/dev/vdd1");
    s.save_configuration(policy(true, 80).replace(&format!("UUID={MAIN}"), "/dev/vdc1"))
        .unwrap();
    let rows = work::checks(&s, &requirements());
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("Fresh swap-policy test"))
    );
    assert!(
        rows.iter().any(|r| r.0 && r.1.starts_with("Saved primary")),
        "Device alias can be correct now yet wrong after remap"
    );
}
#[test]
fn goals_require_all_source_areas_and_bounded_real_permutations_and_positive_work() {
    let s = state();
    let rs = requirements();
    assert!(work::valid_requirements(&rs, s.inventory()));
    let mut bad = rs.clone();
    bad.retain(|r| {
        !matches!(
            r,
            work::Requirement::Swap {
                requirement: swap::Requirement::Fresh { .. }
            }
        )
    });
    assert!(!work::valid_requirements(&bad, s.inventory()));
    let mut bad = rs.clone();
    bad.push(rs[5].clone());
    assert!(!work::valid_requirements(&bad, s.inventory()));
    let mut bad = rs.clone();
    if let work::Requirement::Swap {
        requirement: swap::Requirement::Active { priority, .. },
    } = &mut bad[6]
    {
        *priority = 32768;
    }
    assert!(!work::valid_requirements(&bad, s.inventory()));
    let mut bad = rs.clone();
    if let work::Requirement::Swap {
        requirement: swap::Requirement::Fresh { devices, .. },
    } = bad.last_mut().unwrap()
    {
        devices.insert("/dev/unknown".into(), "primary".into());
    }
    assert!(!work::valid_requirements(&bad, s.inventory()));
    let empty = rs
        .iter()
        .filter(|r| !matches!(r, work::Requirement::Swap { .. }))
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        !work::valid_requirements(&empty, s.inventory()),
        "An enabled capability must have actual outcomes"
    );
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/storage-v1.json")).unwrap();
    let old: storage::Inventory =
        serde_json::from_value(raw["scenarios"][0]["initial"]["storage"]["inventory"].clone())
            .unwrap();
    assert!(!work::valid_requirements(&rs, &old));
}
#[test]
fn literal_commands_disclose_supported_scope_and_refuse_escapes_without_mutation() {
    let mut s = state();
    assert!(
        work::command(&mut s, "help")
            .unwrap()
            .contains("swapon -p N")
    );
    assert!(
        work::command(&mut s, "blkid")
            .unwrap()
            .contains("TYPE=\"swap\"")
    );
    assert!(
        work::command(&mut s, "blkid /dev/vdc1")
            .unwrap()
            .contains(MAIN)
    );
    assert!(
        work::command(&mut s, "swapon --show --bytes")
            .unwrap()
            .contains("1073741824 0 10")
    );
    for c in [
        "swapon /dev/vdc1",
        "swapon --all",
        "swapon -p -1 /dev/vdc1",
        "swapon -p 32768 /dev/vdc1",
        "swapon -p 20 /work/private",
        "swapon -p 20 /dev/../vdc1",
        "swapoff -a",
        "swapoff /dev/vdc1; echo escape",
        "mount -a",
        "sh script",
        "swapon -p 20 /dev/vdc1 | shell",
    ] {
        let before = s.clone();
        assert!(work::command(&mut s, c).is_err(), "{c}");
        assert_eq!(s, before, "{c}");
    }
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/storage-v1.json")).unwrap();
    let mut old: storage::State =
        serde_json::from_value(raw["scenarios"][0]["initial"]["storage"].clone()).unwrap();
    let before = old.clone();
    assert!(work::command(&mut old, "swapon --show --bytes").is_err());
    assert_eq!(old, before);
    assert_eq!(work::command(&mut old, "help").unwrap(), work::HELP);
}
