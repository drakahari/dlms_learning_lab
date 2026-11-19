//! Pure virtual caller/policy/state checks. No host mount, UID, file or process.
use dlms_learning_lab::{removable_mount as old, storage as s, user_mounts as u};
use std::collections::BTreeMap;
const DATA: &str = "00000000-0000-4000-8000-000000000001";
const MEDIA: &str = "00000000-0000-4000-8000-000000000002";
const PERSONAL: &str = "00000000-0000-4000-8000-000000000003";
const TARGET: &str = "/media/audit";
fn environment(present: &[&str]) -> old::Environment {
    old::Environment {
        inventory: s::Inventory::new(
            BTreeMap::from([
                (
                    "data".into(),
                    s::Volume {
                        uuid: DATA.into(),
                        filesystem: s::Filesystem::Xfs,
                    },
                ),
                (
                    "media".into(),
                    s::Volume {
                        uuid: MEDIA.into(),
                        filesystem: s::Filesystem::Ext4,
                    },
                ),
                (
                    "personal".into(),
                    s::Volume {
                        uuid: PERSONAL.into(),
                        filesystem: s::Filesystem::Ext4,
                    },
                ),
            ]),
            BTreeMap::from([
                ("/dev/vda1".into(), "data".into()),
                ("/dev/vdb1".into(), "media".into()),
                ("/dev/vdc1".into(), "personal".into()),
            ]),
            vec!["/project".into(), TARGET.into(), "/media/personal".into()],
        )
        .unwrap(),
        present: present.iter().map(|v| (*v).into()).collect(),
    }
}
fn all() -> old::Environment {
    environment(&["data", "media", "personal"])
}
fn config(options: &str) -> String {
    format!(
        "# Keep required data and a separate personal entry\nUUID={DATA} /project xfs rw,auto 0 2\nUUID={MEDIA} {TARGET} ext4 {options} 0 0\nUUID={PERSONAL} /media/personal ext4 ro,noauto,user 0 0\n"
    )
}
fn policy(options: &str) -> u::Policy {
    u::Policy::parse(&config(options)).unwrap()
}
fn start(options: &str) -> u::Live {
    policy(options).startup(&all()).unwrap().live
}
fn restricted() -> u::Flags {
    u::Flags {
        noexec: true,
        nosuid: true,
        nodev: true,
    }
}
fn rejected(
    live: &mut u::Live,
    env: &old::Environment,
    actor: u::Actor,
    target: &str,
    unmount: bool,
    kind: u::ErrorKind,
) {
    let before = live.clone();
    let error = if unmount {
        live.unmount_as(env, actor, target).unwrap_err()
    } else {
        live.mount_as(env, actor, target).unwrap_err()
    };
    assert_eq!(error.kind, kind, "{}", error.message);
    assert_eq!(
        *live, before,
        "A refused action must not mutate mounts, policy or ledger"
    );
}
#[test]
fn parsed_permission_is_distinct_from_startup_and_old_parser_is_unchanged() {
    for (options, permission) in [
        ("ro,noauto", u::Permission::RootOnly),
        ("ro,noauto,nouser", u::Permission::RootOnly),
        ("ro,noauto,user", u::Permission::MounterOnly),
        ("ro,noauto,users", u::Permission::AnyUser),
    ] {
        let p = policy(options);
        let r = p.rule(TARGET).unwrap();
        assert_eq!(r.permission, permission);
        assert_eq!(r.entry.dependency(), old::Dependency::NotEnrolled);
        assert!(r.entry.mount.read_only);
        assert_eq!(
            r.flags,
            if permission == u::Permission::RootOnly {
                u::Flags::default()
            } else {
                restricted()
            }
        );
        assert_eq!(p.configuration(), config(options));
    }
    for options in ["user", "users", "nouser", "noexec", "nosuid", "nodev"] {
        assert_eq!(
            old::Policy::parse(&config(options)).unwrap_err().kind,
            s::ErrorKind::Unsupported
        );
        assert_eq!(
            s::parse(&config(options)).unwrap_err().kind,
            s::ErrorKind::Unsupported
        );
    }
    let old_raw = config("ro,noauto").replace(",user", "");
    let original = old::Policy::parse(&old_raw).unwrap();
    let overlay = u::Policy::parse(&old_raw).unwrap();
    assert_eq!(overlay.core_policy(), &original);
    assert_eq!(
        overlay.core_policy().startup(&all()).unwrap(),
        original.startup(&all()).unwrap()
    );
}
#[test]
fn implied_restrictions_follow_actual_token_order_and_defaults_does_not_reset_them() {
    for (options, flags) in [
        (
            "user,exec",
            u::Flags {
                noexec: false,
                ..restricted()
            },
        ),
        ("exec,user", restricted()),
        ("user,defaults", restricted()),
        ("noexec,nosuid,nodev,nouser", restricted()),
        (
            "users,dev,exec,suid,nodev,noexec",
            u::Flags {
                noexec: true,
                nosuid: false,
                nodev: true,
            },
        ),
        ("nouser,exec,suid,dev", u::Flags::default()),
    ] {
        assert_eq!(
            policy(options).rule(TARGET).unwrap().flags,
            flags,
            "{options}"
        );
    }
    let p = policy("rw,noauto,ro,auto,user,nofail");
    let r = p.rule(TARGET).unwrap();
    assert!(r.entry.mount.read_only && r.entry.mount.automatic && r.entry.optional);
    assert_eq!(r.entry.dependency(), old::Dependency::Wanted);
}
#[test]
fn unsupported_permission_combinations_and_options_fail_without_silent_approximation() {
    for options in [
        "user,users",
        "users,nouser",
        "user,nouser",
        "nouser,user",
        "user,user",
        "nouser,nouser",
        "user=first",
        "owner",
        "group",
        "nousers",
        "uid=1000",
        "gid=1000",
        "x-systemd.automount",
        "unknown",
        ",user",
        "user,",
    ] {
        let e = u::Policy::parse(&config(options)).unwrap_err();
        assert_eq!(e.kind, u::ErrorKind::Unsupported, "{options}: {e}");
        assert_eq!(e.line, Some(3));
    }
    assert_eq!(
        u::Policy::parse(&" ".repeat(s::MAX_CONFIG_BYTES + 1))
            .unwrap_err()
            .kind,
        u::ErrorKind::Limit
    );
    assert_eq!(policy("user").rules().len(), 3);
    assert_eq!(
        u::Policy::parse(&config(&vec!["noexec"; 17].join(",")))
            .unwrap_err()
            .kind,
        u::ErrorKind::Limit
    );
    for raw in [
        config("user").replace(TARGET, "/media/../audit"),
        config("user").replace(TARGET, "/project"),
        config("user").replace("0 2", "0"),
        config("user") + "\0",
    ] {
        assert!(u::Policy::parse(&raw).is_err());
    }
}
#[test]
fn user_allows_each_mounting_caller_but_only_the_real_mounter_can_unmount() {
    let e = all();
    let mut live = start("ro,noauto,user");
    assert_eq!(
        live.mount_as(&e, u::Actor::First, TARGET).unwrap(),
        s::MountOutcome::Mounted
    );
    assert_eq!(live.ledger()[TARGET].mounter, Some(u::Actor::First));
    assert_eq!(live.ledger()[TARGET].flags, Some(restricted()));
    let before = live.clone();
    assert_eq!(
        live.mount_as(&e, u::Actor::Second, TARGET).unwrap(),
        s::MountOutcome::AlreadyMounted
    );
    assert_eq!(
        live, before,
        "An existing mount cannot transfer its actual mounter"
    );
    rejected(
        &mut live,
        &e,
        u::Actor::Second,
        TARGET,
        true,
        u::ErrorKind::PermissionDenied,
    );
    live.unmount_as(&e, u::Actor::First, TARGET).unwrap();
    assert!(!live.state().live().contains_key(TARGET) && !live.ledger().contains_key(TARGET));
    live.mount_as(&e, u::Actor::Second, TARGET).unwrap();
    assert_eq!(live.ledger()[TARGET].mounter, Some(u::Actor::Second));
    rejected(
        &mut live,
        &e,
        u::Actor::First,
        TARGET,
        true,
        u::ErrorKind::PermissionDenied,
    );
    live.unmount_as(&e, u::Actor::Root, TARGET).unwrap();
    assert!(live.state().live().contains_key("/project"));
}
#[test]
fn users_allows_handover_without_changing_the_personal_sibling_policy() {
    let e = all();
    let mut live = start("ro,noauto,users");
    for mounter in [u::Actor::First, u::Actor::Second, u::Actor::Root] {
        live.mount_as(&e, mounter, TARGET).unwrap();
        live.unmount_as(&e, u::Actor::Second, TARGET).unwrap();
    }
    live.mount_as(&e, u::Actor::First, "/media/personal")
        .unwrap();
    rejected(
        &mut live,
        &e,
        u::Actor::Second,
        "/media/personal",
        true,
        u::ErrorKind::PermissionDenied,
    );
    rejected(
        &mut live,
        &e,
        u::Actor::First,
        "/project",
        false,
        u::ErrorKind::PermissionDenied,
    );
    rejected(
        &mut live,
        &e,
        u::Actor::First,
        "/project",
        true,
        u::ErrorKind::PermissionDenied,
    );
}
#[test]
fn unknown_imported_history_is_not_invented_as_root_or_an_ordinary_mounter() {
    let e = all();
    let p = policy("ro,noauto,user");
    let mounts = BTreeMap::from([(
        TARGET.into(),
        s::Mount {
            volume: "media".into(),
            read_only: true,
        },
    )]);
    let mut live = u::Live::new(&e, &p, mounts).unwrap();
    assert_eq!(live.ledger()[TARGET].mounter, None);
    assert_eq!(live.ledger()[TARGET].flags, None);
    rejected(
        &mut live,
        &e,
        u::Actor::First,
        TARGET,
        true,
        u::ErrorKind::PermissionDenied,
    );
    let before = live.clone();
    assert_eq!(
        live.mount_as(&e, u::Actor::First, TARGET).unwrap(),
        s::MountOutcome::AlreadyMounted
    );
    assert_eq!(
        live, before,
        "Already-mounted cannot invent unknown history"
    );
    live.save_policy(&policy("ro,noauto,users")).unwrap();
    live.unmount_as(&e, u::Actor::Second, TARGET).unwrap();
}
#[test]
fn saved_edits_do_not_remount_rewrite_flags_or_transfer_ownership() {
    let e = all();
    let mut live = start("ro,noauto,user");
    live.mount_as(&e, u::Actor::First, TARGET).unwrap();
    let applied = live.ledger()[TARGET].clone();
    live.save_policy(&policy("ro,noauto,users,exec")).unwrap();
    assert_eq!(live.ledger()[TARGET], applied);
    assert_eq!(
        live.mount_as(&e, u::Actor::Second, TARGET).unwrap(),
        s::MountOutcome::AlreadyMounted
    );
    assert_eq!(live.ledger()[TARGET], applied);
    live.unmount_as(&e, u::Actor::Second, TARGET).unwrap();
    live.mount_as(&e, u::Actor::Second, TARGET).unwrap();
    assert_eq!(live.ledger()[TARGET].mounter, Some(u::Actor::Second));
    assert_eq!(
        live.ledger()[TARGET].flags,
        Some(u::Flags {
            noexec: false,
            ..restricted()
        })
    );
    live.save_policy(&policy("ro,noauto,nouser")).unwrap();
    rejected(
        &mut live,
        &e,
        u::Actor::Second,
        TARGET,
        true,
        u::ErrorKind::PermissionDenied,
    );
    live.unmount_as(&e, u::Actor::Root, TARGET).unwrap();
    rejected(
        &mut live,
        &e,
        u::Actor::First,
        TARGET,
        false,
        u::ErrorKind::PermissionDenied,
    );
    live.mount_as(&e, u::Actor::Root, TARGET).unwrap();
    assert_eq!(live.ledger()[TARGET].mounter, Some(u::Actor::Root));
}
#[test]
fn saved_readonly_changes_and_current_source_pair_are_not_confused_with_live_state() {
    let e = all();
    let mut live = start("ro,noauto,user");
    live.mount_as(&e, u::Actor::First, TARGET).unwrap();
    live.save_policy(&policy("rw,noauto,user,exec")).unwrap();
    rejected(
        &mut live,
        &e,
        u::Actor::First,
        TARGET,
        false,
        u::ErrorKind::Occupied,
    );
    assert!(live.state().live()[TARGET].read_only);
    assert_eq!(live.ledger()[TARGET].flags, Some(restricted()));
    let wrong_pair = u::Policy::parse(&config("ro,noauto,user").replace(MEDIA, PERSONAL)).unwrap();
    live.save_policy(&wrong_pair).unwrap();
    rejected(
        &mut live,
        &e,
        u::Actor::First,
        TARGET,
        true,
        u::ErrorKind::Mismatch,
    );
    live.unmount_as(&e, u::Actor::Root, TARGET).unwrap();
    let mut live = start("ro,noauto,user");
    live.mount_as(&e, u::Actor::First, TARGET).unwrap();
    let removed = config("ro,noauto,user")
        .lines()
        .filter(|line| !line.contains(TARGET))
        .collect::<Vec<_>>()
        .join("\n");
    live.save_policy(&u::Policy::parse(&removed).unwrap())
        .unwrap();
    rejected(
        &mut live,
        &e,
        u::Actor::First,
        TARGET,
        true,
        u::ErrorKind::Mismatch,
    );
    live.unmount_as(&e, u::Actor::Root, TARGET).unwrap();
}
#[test]
fn absence_unknown_source_bad_type_and_foreign_context_are_distinct_and_atomic() {
    let absent = environment(&["data", "personal"]);
    let p = policy("ro,noauto,user,nofail");
    let mut live = p.startup(&absent).unwrap().live;
    rejected(
        &mut live,
        &absent,
        u::Actor::First,
        TARGET,
        false,
        u::ErrorKind::Unavailable,
    );
    let e = all();
    let mut live = start("ro,noauto,user");
    let unknown = u::Policy::parse(
        &config("ro,noauto,user").replace(MEDIA, "00000000-0000-4000-8000-000000000099"),
    )
    .unwrap();
    live.save_policy(&unknown).unwrap();
    rejected(
        &mut live,
        &e,
        u::Actor::Root,
        TARGET,
        false,
        u::ErrorKind::UnknownSource,
    );
    live.save_policy(
        &u::Policy::parse(
            &config("ro,noauto,user").replace("/media/audit ext4", "/media/audit xfs"),
        )
        .unwrap(),
    )
    .unwrap();
    rejected(
        &mut live,
        &e,
        u::Actor::Root,
        TARGET,
        false,
        u::ErrorKind::TypeMismatch,
    );
    rejected(
        &mut live,
        &e,
        u::Actor::Root,
        "/home/personal",
        false,
        u::ErrorKind::Invalid,
    );
    rejected(
        &mut live,
        &e,
        u::Actor::Root,
        "/media/../audit",
        true,
        u::ErrorKind::Invalid,
    );
    rejected(
        &mut live,
        &e,
        u::Actor::Root,
        "/media/personal",
        true,
        u::ErrorKind::NotMounted,
    );
    let mut aliases = e.inventory.devices().clone();
    aliases.insert("/dev/vda1".into(), "media".into());
    aliases.insert("/dev/vdb1".into(), "data".into());
    let foreign = old::Environment {
        inventory: s::Inventory::new(
            e.inventory.volumes().clone(),
            aliases,
            e.inventory.mountpoints().iter().cloned().collect(),
        )
        .unwrap(),
        present: e.present.clone(),
    };
    rejected(
        &mut live,
        &foreign,
        u::Actor::Root,
        TARGET,
        false,
        u::ErrorKind::Mismatch,
    );
    let changed = policy("ro,noauto,user").startup(&foreign).unwrap();
    assert_eq!(changed.live.state().live()["/project"].volume, "data");
}
#[test]
fn startup_records_only_actual_privileged_mounts_and_refuses_hot_removal() {
    let e = all();
    let p = policy("ro,auto,user,nofail");
    let startup = p.startup(&e).unwrap();
    assert!(startup.required_attempts_satisfied());
    assert_eq!(
        startup.live.ledger()["/project"].mounter,
        Some(u::Actor::Root)
    );
    assert_eq!(startup.live.ledger()[TARGET].mounter, Some(u::Actor::Root));
    assert_eq!(startup.live.ledger()[TARGET].flags, Some(restricted()));
    assert!(!startup.live.ledger().contains_key("/media/personal"));
    assert_eq!(startup.live.configuration(), p.configuration());
    assert_eq!(
        startup.live.state().configuration(),
        p.core_policy().core_configuration()
    );
    let missing = environment(&["data", "personal"]);
    let mut live = startup.live;
    rejected(
        &mut live,
        &missing,
        u::Actor::Root,
        TARGET,
        true,
        u::ErrorKind::Unsupported,
    );
    let partial = p.startup(&missing).unwrap();
    assert!(partial.required_attempts_satisfied());
    assert!(!partial.live.state().live().contains_key(TARGET));
    let missing_required = p.startup(&environment(&["media", "personal"])).unwrap();
    assert!(!missing_required.required_attempts_satisfied());
}
#[test]
fn bounded_snapshot_roundtrip_preserves_raw_saved_text_actual_flags_and_old_core_wire() {
    let e = all();
    let mut live = start("ro,noauto,user");
    live.mount_as(&e, u::Actor::First, TARGET).unwrap();
    live.save_policy(&policy("ro,noauto,users,exec")).unwrap();
    let raw = serde_json::to_string(&live).unwrap();
    let restored = u::Live::from_json(&raw).unwrap();
    assert_eq!(restored, live);
    restored.validate_for(&e).unwrap();
    assert_eq!(restored.ledger()[TARGET].flags, Some(restricted()));
    let old_wire = serde_json::to_value(restored.state()).unwrap();
    assert_eq!(
        old_wire["live"][TARGET],
        serde_json::json!({"volume":"media","read_only":true})
    );
    assert!(
        old_wire["configuration"]
            .as_str()
            .unwrap()
            .contains("ro,noauto")
    );
    assert!(
        !old_wire["configuration"]
            .as_str()
            .unwrap()
            .contains("users")
    );
    assert_eq!(
        serde_json::from_value::<s::State>(old_wire).unwrap(),
        *restored.state()
    );
    assert_eq!(
        u::Live::from_json(&" ".repeat(u::MAX_STATE_BYTES + 1))
            .unwrap_err()
            .kind,
        u::ErrorKind::Limit
    );
}
#[test]
fn malformed_or_inconsistent_ledger_and_nested_state_are_refused_during_decode() {
    let e = all();
    let mut live = start("ro,noauto,user");
    live.mount_as(&e, u::Actor::First, TARGET).unwrap();
    let original = serde_json::to_value(&live).unwrap();
    for n in 0..9 {
        let mut bad = original.clone();
        match n {
            0 => bad["version"] = 2.into(),
            1 => bad["ledger"][TARGET]["volume"] = "personal".into(),
            2 => bad["ledger"][TARGET]["mounter"] = "foreign".into(),
            3 => bad["ledger"][TARGET]["flags"] = serde_json::Value::Null,
            4 => bad["ledger"]
                .as_object_mut()
                .unwrap()
                .remove(TARGET)
                .map(|_| ())
                .unwrap(),
            5 => {
                bad["configuration"] =
                    (live.configuration().to_owned() + "# changed saved policy\n").into()
            }
            6 => bad["state"]["configuration"] = "different core configuration".into(),
            7 => bad["state"]["live"][TARGET]["volume"] = "x".repeat(49).into(),
            _ => bad["unknown"] = true.into(),
        }
        assert!(
            serde_json::from_value::<u::Live>(bad).is_err(),
            "tamper case {n}"
        );
    }
    let mut too_many = original.clone();
    let rows = too_many["ledger"].as_object_mut().unwrap();
    for n in 0..9 {
        rows.insert(
            format!("/extra/{n}"),
            serde_json::json!({"volume":"media","mounter":null,"flags":null}),
        );
    }
    assert!(serde_json::from_value::<u::Live>(too_many).is_err());
    let mut oversized = original.clone();
    oversized["configuration"] = " ".repeat(s::MAX_CONFIG_BYTES + 1).into();
    assert!(serde_json::from_value::<u::Live>(oversized).is_err());
    let mut nested = original;
    nested["state"]["configuration"] = " ".repeat(s::MAX_CONFIG_BYTES + 1).into();
    assert!(serde_json::from_value::<u::Live>(nested).is_err());
}
