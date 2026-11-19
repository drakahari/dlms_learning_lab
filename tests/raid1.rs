//! Pure synthetic RAID1 state tests; never a host mdadm/device execution.
use dlms_learning_lab::raid1::*;
#[path = "support/raid1.rs"]
mod support;
use support::*;
#[test]
fn inspection_success_is_not_redundancy_and_data_is_read_from_a_real_replica() {
    let mut w = world();
    w.validate().unwrap();
    let old = w.clone();
    let out = w.execute("mdadm --detail /dev/md0").unwrap();
    assert_eq!(out.status, 0);
    assert!(out.output.contains("Degraded"));
    assert_eq!(w, old);
    assert_eq!(w.health("/dev/md0").unwrap(), Health::Degraded);
    assert_eq!(
        w.read_file("/dev/md0", "/records.txt").unwrap(),
        data("records")
            .iter()
            .map(|p| p.text.as_str())
            .collect::<String>()
    );
    assert!(w.read_file("/dev/md0", "/../records.txt").is_err());
    assert!(w.read_file("/dev/md0", "/absent").is_err());
}
#[test]
fn remove_active_and_last_source_failure_refuse_without_any_publication() {
    let mut w = world();
    let old = w.clone();
    let out = w.execute("mdadm /dev/md0 --remove /dev/vda1").unwrap();
    assert_eq!(out.status, 1);
    assert!(out.output.contains("active"));
    assert_eq!(w, old);
    let err = w.execute("mdadm /dev/md0 --fail /dev/vda1").unwrap_err();
    assert_eq!(err.kind, ErrorKind::Unsupported);
    assert!(err.message.contains("last complete source"));
    assert_eq!(w, old);
}
#[test]
fn actual_copying_partial_resume_and_completion_keep_source_and_neighbors() {
    let initial = world();
    let mut w = ready("/dev/vdc1");
    assert_eq!(w.devices["/dev/vdb1"], initial.devices["/dev/vdb1"]);
    assert_eq!(
        w.health("/dev/md0").unwrap(),
        Health::Rebuilding {
            copied: 0,
            total: 8
        }
    );
    assert!(w.devices["/dev/vdc1"].content.is_empty());
    w.apply(&Action::Advance {
        array: "/dev/md0".into(),
        units: 3,
    })
    .unwrap();
    assert_eq!(w.devices["/dev/vdc1"].content, &data("records")[..3]);
    assert_eq!(
        w.health("/dev/md0").unwrap(),
        Health::Rebuilding {
            copied: 3,
            total: 8
        }
    );
    w = State::from_json(&w.to_json().unwrap()).unwrap();
    assert!(w.environment_matches(&initial));
    w.apply(&Action::Advance {
        array: "/dev/md0".into(),
        units: 128,
    })
    .unwrap();
    assert_eq!(w.health("/dev/md0").unwrap(), Health::Healthy);
    assert_eq!(w.devices["/dev/vdc1"].content, data("records"));
    assert_eq!(w.devices["/dev/vda1"], initial.devices["/dev/vda1"]);
    assert_eq!(w.devices["/dev/vdg1"], initial.devices["/dev/vdg1"]);
    assert_eq!(w.arrays["/dev/md1"], initial.arrays["/dev/md1"]);
    cmd(&mut w, "mdadm /dev/md0 --fail /dev/vdc1");
    assert_eq!(w.health("/dev/md0").unwrap(), Health::Degraded);
}
#[test]
fn missing_slots_do_not_require_removing_an_imaginary_member() {
    let mut w = world();
    w.arrays.get_mut("/dev/md0").unwrap().members.pop();
    w.validate().unwrap();
    let old = w.clone();
    let out = w.execute("mdadm /dev/md0 --remove /dev/vdb1").unwrap();
    assert_eq!(out.status, 1);
    assert!(out.output.contains("missing slot"));
    assert_eq!(w, old);
    cmd(&mut w, "mdadm /dev/md0 --add /dev/vdc1");
    w.apply(&Action::Advance {
        array: "/dev/md0".into(),
        units: 128,
    })
    .unwrap();
    assert_eq!(w.health("/dev/md0").unwrap(), Health::Healthy);
}
#[test]
fn equivalent_sufficient_fresh_media_and_supported_command_forms_reach_equal_data() {
    for device in ["/dev/vdc1", "/dev/vdd1"] {
        let mut w = world();
        cmd(&mut w, "mdadm --manage /dev/md0 --remove /dev/vdb1");
        cmd(&mut w, &format!("mdadm --manage /dev/md0 --add {device}"));
        w.apply(&Action::Advance {
            array: "/dev/md0".into(),
            units: 8,
        })
        .unwrap();
        assert_eq!(w.health("/dev/md0").unwrap(), Health::Healthy);
        assert_eq!(w.devices[device].content, data("records"));
        assert_eq!(w.execute("mdadm -D /dev/md0").unwrap().status, 0);
    }
}
#[test]
fn usable_capacity_protection_and_media_refusals_are_specific_and_atomic() {
    for (candidate, expected) in [
        ("/dev/vde1", "usable MD capacity"),
        ("/dev/vdf1", "fresh blank"),
        ("/dev/vdg1", "protected"),
    ] {
        let mut w = world();
        let old = w.clone();
        let text = match w.execute(&format!("mdadm /dev/md0 --add {candidate}")) {
            Ok(o) => {
                assert_ne!(o.status, 0);
                o.output
            }
            Err(e) => e.message,
        };
        assert!(text.contains(expected), "{text}");
        assert_eq!(w, old);
    }
    for (flag, expected) in [("available", "available"), ("read_only", "read-only")] {
        let mut w = world();
        let d = w.devices.get_mut("/dev/vdc1").unwrap();
        if flag == "available" {
            d.available = false
        } else {
            d.read_only = true
        }
        let old = w.clone();
        let out = w.execute("mdadm /dev/md0 --add /dev/vdc1").unwrap();
        assert_eq!(out.status, 1);
        assert!(out.output.contains(expected));
        assert_eq!(w, old);
    }
}
#[test]
fn claimed_percentages_and_disagreeing_complete_replicas_cannot_validate() {
    let w = ready("/dev/vdc1");
    let mut bad = w.clone();
    bad.arrays
        .get_mut("/dev/md0")
        .unwrap()
        .members
        .last_mut()
        .unwrap()
        .role = Role::Recovering { copied: 3 };
    assert!(bad.validate().is_err());
    let mut bad = w.clone();
    bad.devices.get_mut("/dev/vdc1").unwrap().content = data("wrong")[..3].to_vec();
    bad.arrays
        .get_mut("/dev/md0")
        .unwrap()
        .members
        .last_mut()
        .unwrap()
        .role = Role::Recovering { copied: 3 };
    assert!(bad.validate().is_err());
    let mut bad = w;
    bad.devices.get_mut("/dev/vdc1").unwrap().content = data("wrong");
    bad.arrays
        .get_mut("/dev/md0")
        .unwrap()
        .members
        .last_mut()
        .unwrap()
        .role = Role::Synced;
    assert!(bad.validate().is_err());
}
#[test]
fn healthy_spare_is_not_a_third_mirror_and_sole_spare_recovers_after_failure() {
    let mut w = world();
    w.devices.get_mut("/dev/vdb1").unwrap().content = data("records");
    w.arrays.get_mut("/dev/md0").unwrap().members[1].role = Role::Synced;
    w.validate().unwrap();
    cmd(&mut w, "mdadm /dev/md0 --add /dev/vdc1");
    assert_eq!(w.health("/dev/md0").unwrap(), Health::Healthy);
    assert_eq!(w.arrays["/dev/md0"].members[2].role, Role::Spare);
    assert!(w.devices["/dev/vdc1"].content.is_empty());
    let mut removed = w.clone();
    cmd(&mut removed, "mdadm /dev/md0 --remove /dev/vdc1");
    assert!(matches!(
        removed.devices["/dev/vdc1"].media,
        Media::Md { .. }
    ));
    assert_eq!(removed.health("/dev/md0").unwrap(), Health::Healthy);
    cmd(&mut w, "mdadm /dev/md0 --fail /dev/vdb1");
    assert_eq!(
        w.health("/dev/md0").unwrap(),
        Health::Rebuilding {
            copied: 0,
            total: 8
        }
    );
    w.apply(&Action::Advance {
        array: "/dev/md0".into(),
        units: 8,
    })
    .unwrap();
    assert_eq!(w.health("/dev/md0").unwrap(), Health::Healthy);
    let old = w.clone();
    assert!(
        w.execute("mdadm /dev/md0 --fail /dev/vda1")
            .unwrap_err()
            .message
            .contains("Remove the old failed")
    );
    assert_eq!(w, old);
}
#[test]
fn unsupported_syntax_and_escape_strings_never_fall_through_to_a_host() {
    for c in [
        "sh -c id",
        "mdadm --create /dev/md0",
        "mdadm --detail --test /dev/md0",
        "mdadm /dev/md0 --replace /dev/vda1",
        "mdadm /dev/md0 --add /dev/../sda",
        "mdadm /dev/md0 --add $(id)",
        "mdadm /dev/md0 --add /dev/vdc1;id",
        "mdadm /dev/md0 --add /dev/vdc1\nwhoami",
    ] {
        let mut w = world();
        let old = w.clone();
        assert!(w.execute(c).is_err(), "{c}");
        assert_eq!(w, old);
    }
}
#[test]
fn encoded_limits_duplicate_keys_and_closed_actions_are_enforced() {
    let w = world();
    let raw = w.to_json().unwrap();
    assert_eq!(State::from_json(&raw).unwrap(), w);
    assert!(State::from_json(&" ".repeat(MAX_STATE_BYTES + 1)).is_err());
    assert!(
        State::from_json(&raw.replacen("\"version\":1", "\"version\":1,\"version\":1", 1)).is_err()
    );
    let mut bad = w.clone();
    bad.devices
        .insert("/dev/extra".into(), disk(Media::Blank, vec![]));
    assert!(bad.validate().is_err());
    let mut bad = w;
    bad.devices.get_mut("/dev/vdc1").unwrap().content = vec![Segment {
        file: "/x".into(),
        index: 0,
        text: "x".repeat(257),
    }];
    assert!(bad.validate().is_err());
    for text in [
        r#"{"kind":"add","array":"/dev/md0","device":"/dev/vdc1","units":1}"#,
        r#"{"kind":"advance","array":"/dev/md0","units":1,"extra":{}}"#,
        r#"{"kind":"add","kind":"add","array":"/dev/md0","device":"/dev/vdc1"}"#,
    ] {
        assert!(serde_json::from_str::<Action>(text).is_err());
    }
    let a = Action::Advance {
        array: "/dev/md0".into(),
        units: 3,
    };
    assert_eq!(
        serde_json::from_str::<Action>(&serde_json::to_string(&a).unwrap()).unwrap(),
        a
    );
}
#[test]
fn immutable_inventory_changes_and_stale_metadata_readd_are_not_accepted() {
    let initial = world();
    let mut w = ready("/dev/vdc1");
    assert!(w.environment_matches(&initial));
    w.devices.get_mut("/dev/vdc1").unwrap().usable_sectors += 1;
    assert!(!w.environment_matches(&initial));
    let mut w = ready("/dev/vdc1");
    let old = w.clone();
    assert!(
        w.execute("mdadm /dev/md0 --add /dev/vdb1")
            .unwrap_err()
            .message
            .contains("re-add")
    );
    assert_eq!(w, old);
}
#[test]
fn zero_excess_and_unstarted_recovery_have_bounded_noop_failures() {
    for units in [0, 129, u16::MAX] {
        let mut w = ready("/dev/vdc1");
        let old = w.clone();
        assert!(
            w.apply(&Action::Advance {
                array: "/dev/md0".into(),
                units
            })
            .is_err()
        );
        assert_eq!(w, old);
    }
    let mut w = world();
    let old = w.clone();
    assert!(
        w.apply(&Action::Advance {
            array: "/dev/md0".into(),
            units: 1
        })
        .is_err()
    );
    assert_eq!(w, old);
}
