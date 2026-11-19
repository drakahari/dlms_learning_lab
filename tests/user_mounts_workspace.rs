//! Behavior and provenance checks for actor-aware virtual mount assignments.
use dlms_learning_lab::{
    removable_mount::Environment, storage as s, user_mounts as m, user_mounts_workspace as w,
};
use std::collections::BTreeMap;
const DATA: &str = "00000000-0000-4000-8000-000000000001";
const MEDIA: &str = "00000000-0000-4000-8000-000000000002";
const PERSONAL: &str = "00000000-0000-4000-8000-000000000003";
const TARGET: &str = "/media/audit";
fn environment() -> Environment {
    Environment {
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
        present: ["data".into(), "media".into(), "personal".into()]
            .into_iter()
            .collect(),
    }
}
fn config(options: &str) -> String {
    format!(
        "UUID={DATA} /project xfs rw,auto 0 2\nUUID={MEDIA} {TARGET} ext4 {options} 0 0\nUUID={PERSONAL} /media/personal ext4 ro,noauto,user 0 0\n"
    )
}
fn restricted() -> m::Flags {
    m::Flags {
        noexec: true,
        nosuid: true,
        nodev: true,
    }
}
fn mount(actor: m::Actor, target: &str) -> w::Action {
    w::Action::Mount {
        actor,
        target: target.into(),
    }
}
fn unmount(actor: m::Actor, target: &str) -> w::Action {
    w::Action::Unmount {
        actor,
        target: target.into(),
    }
}
fn allowed(action: w::Action, effect: w::Effect) -> w::Probe {
    w::Probe {
        action,
        expected: w::Decision::Succeeded { effect },
    }
}
fn denied(action: w::Action) -> w::Probe {
    w::Probe {
        action,
        expected: w::Decision::Refused {
            reason: m::ErrorKind::PermissionDenied,
        },
    }
}
fn live(target: &str, volume: &str, actor: m::Actor, flags: m::Flags, ro: bool) -> w::Requirement {
    w::Requirement::Mounted {
        target: target.into(),
        volume: volume.into(),
        read_only: ro,
        mounter: actor,
        flags,
    }
}
fn task(shared: bool) -> w::Task {
    use m::Actor::*;
    use w::Effect::*;
    let env = environment();
    let protected = live("/project", "data", Root, m::Flags::default(), false);
    let personal = live("/media/personal", "personal", First, restricted(), true);
    let seed = if shared {
        vec![mount(First, TARGET), mount(First, "/media/personal")]
    } else {
        vec![]
    };
    let permission = if shared {
        m::Permission::AnyUser
    } else {
        m::Permission::MounterOnly
    };
    let mut requirements = vec![
        w::Requirement::Policy {
            target: TARGET.into(),
            volume: "media".into(),
            read_only: true,
            automatic: false,
            optional: false,
            permission,
            flags: restricted(),
        },
        w::Requirement::Policy {
            target: "/project".into(),
            volume: "data".into(),
            read_only: false,
            automatic: true,
            optional: false,
            permission: m::Permission::RootOnly,
            flags: m::Flags::default(),
        },
        protected.clone(),
        live(
            TARGET,
            "media",
            if shared { Second } else { First },
            restricted(),
            true,
        ),
    ];
    if shared {
        requirements.push(personal.clone());
    }
    let mut absent = env.clone();
    absent.present.remove("media");
    let cases = if !shared {
        vec![
            w::Case {
                id: "first-mounter".into(),
                label: "First operator owns the mount".into(),
                environment: env.clone(),
                seed: vec![],
                probes: vec![
                    allowed(mount(First, TARGET), Mounted),
                    allowed(mount(Second, TARGET), AlreadyMounted),
                    denied(unmount(Second, TARGET)),
                ],
                final_requirements: vec![
                    live(TARGET, "media", First, restricted(), true),
                    protected.clone(),
                ],
            },
            w::Case {
                id: "second-mounter".into(),
                label: "Second operator may start their own session".into(),
                environment: env.clone(),
                seed: vec![],
                probes: vec![
                    allowed(mount(Second, TARGET), Mounted),
                    denied(unmount(First, TARGET)),
                    allowed(unmount(Second, TARGET), Unmounted),
                ],
                final_requirements: vec![
                    w::Requirement::NotMounted {
                        target: TARGET.into(),
                    },
                    protected.clone(),
                ],
            },
            w::Case {
                id: "administrator".into(),
                label: "Administrator remains a distinct caller".into(),
                environment: env.clone(),
                seed: vec![],
                probes: vec![
                    allowed(mount(Root, TARGET), Mounted),
                    denied(unmount(First, TARGET)),
                    allowed(unmount(Root, TARGET), Unmounted),
                ],
                final_requirements: vec![protected.clone()],
            },
            w::Case {
                id: "missing-medium".into(),
                label: "Permission does not create a missing device".into(),
                environment: absent,
                seed: vec![],
                probes: vec![w::Probe {
                    action: mount(First, TARGET),
                    expected: w::Decision::Refused {
                        reason: m::ErrorKind::Unavailable,
                    },
                }],
                final_requirements: vec![protected],
            },
        ]
    } else {
        vec![
            w::Case {
                id: "shared-handover".into(),
                label: "Shared handover keeps the personal sibling".into(),
                environment: env.clone(),
                seed: seed.clone(),
                probes: vec![
                    allowed(unmount(Second, TARGET), Unmounted),
                    allowed(mount(Second, TARGET), Mounted),
                    denied(unmount(Second, "/media/personal")),
                ],
                final_requirements: vec![
                    live(TARGET, "media", Second, restricted(), true),
                    personal.clone(),
                    protected.clone(),
                ],
            },
            w::Case {
                id: "personal-remains-personal".into(),
                label: "Personal mounter may remount their own volume".into(),
                environment: env.clone(),
                seed: seed.clone(),
                probes: vec![
                    allowed(unmount(First, "/media/personal"), Unmounted),
                    allowed(mount(First, "/media/personal"), Mounted),
                    allowed(unmount(Second, TARGET), Unmounted),
                ],
                final_requirements: vec![
                    personal.clone(),
                    w::Requirement::NotMounted {
                        target: TARGET.into(),
                    },
                    protected.clone(),
                ],
            },
            w::Case {
                id: "shared-absent".into(),
                label: "Missing shared medium does not grant personal access".into(),
                environment: absent,
                seed: vec![mount(First, "/media/personal")],
                probes: vec![
                    w::Probe {
                        action: mount(Second, TARGET),
                        expected: w::Decision::Refused {
                            reason: m::ErrorKind::Unavailable,
                        },
                    },
                    denied(unmount(Second, "/media/personal")),
                ],
                final_requirements: vec![personal, protected],
            },
        ]
    };
    w::Task{id:if shared{"shared-handover"}else{"personal-session"}.into(),instructions:"Repair saved policy, make the required actual mount, preserve protected storage, and satisfy all supplied caller cases.".into(),first_actor:"Alex".into(),second_actor:"Casey".into(),focus_target:TARGET.into(),environment:env,starter:config(if shared{"ro,noauto,user,exec"}else{"ro,noauto,nouser"}),seed,requirements,cases}
}
fn save(work: &mut w::Work, t: &w::Task, options: &str) {
    work.apply(
        t,
        w::Action::Save {
            configuration: config(options),
        },
    )
    .unwrap();
}
fn check(work: &mut w::Work, t: &w::Task) {
    assert!(work.store(w::run(t, work).unwrap(), t));
}
#[test]
fn meaningful_personal_session_and_supported_alternatives_require_real_mounter_state() {
    let t = task(false);
    t.validate_authored().unwrap();
    for options in [
        "ro,noauto,user",
        "user,noauto,ro,nofail,noexec,nosuid,nodev",
        "defaults,ro,noauto,nodev,user",
    ] {
        let mut work = w::Work::new(&t);
        save(&mut work, &t, options);
        check(&mut work, &t);
        assert!(
            !work.solved(&t),
            "Saving a policy does not perform the actual mount"
        );
        work.apply(&t, mount(m::Actor::First, TARGET)).unwrap();
        check(&mut work, &t);
        assert!(work.solved(&t), "{options}: {:?}", work.result);
        let before = work.clone();
        assert!(work.apply(&t, unmount(m::Actor::Second, TARGET)).is_err());
        assert_eq!(
            work, before,
            "denied action changes no saved work or credit"
        );
        work.apply(&t, mount(m::Actor::Second, TARGET)).unwrap();
        assert_eq!(
            work, before,
            "already mounted cannot transfer actor, flags or history"
        );
        work.apply(&t, unmount(m::Actor::First, TARGET)).unwrap();
        assert!(!work.solved(&t));
    }
}
#[test]
fn shared_repair_requires_unmount_remount_after_save_and_preserves_personal_sibling() {
    let t = task(true);
    t.validate_authored().unwrap();
    let mut work = w::Work::new(&t);
    save(&mut work, &t, "ro,noauto,users");
    check(&mut work, &t);
    assert!(!work.solved(&t));
    assert!(
        !work.live(&t).unwrap().ledger()[TARGET]
            .flags
            .unwrap()
            .noexec
    );
    work.apply(&t, unmount(m::Actor::Second, TARGET)).unwrap();
    work.apply(&t, mount(m::Actor::Second, TARGET)).unwrap();
    check(&mut work, &t);
    assert!(work.solved(&t), "{:?}", work.result);
    let before = work.clone();
    assert!(
        work.apply(&t, unmount(m::Actor::Second, "/media/personal"))
            .is_err()
    );
    assert_eq!(work, before);
}
#[test]
fn wrong_permission_flags_source_readonly_and_hardcoded_actor_assumptions_fail() {
    for (shared, wrongs) in [
        (
            false,
            vec![
                "ro,noauto,users",
                "ro,noauto,nouser",
                "rw,noauto,user",
                "ro,auto,user",
                "ro,noauto,user,exec",
            ],
        ),
        (
            true,
            vec!["ro,noauto,user", "ro,noauto,users,exec", "rw,noauto,users"],
        ),
    ] {
        let t = task(shared);
        for options in wrongs {
            let mut work = w::Work::new(&t);
            save(&mut work, &t, options);
            if shared {
                let _ = work.apply(&t, unmount(m::Actor::Root, TARGET));
            }
            let _ = work.apply(
                &t,
                mount(
                    if shared {
                        m::Actor::Second
                    } else {
                        m::Actor::First
                    },
                    TARGET,
                ),
            );
            check(&mut work, &t);
            assert!(!work.solved(&t), "false positive {shared} {options}");
        }
    }
    let t = task(false);
    let mut work = w::Work::new(&t);
    save(&mut work, &t, "ro,noauto,user");
    work.apply(&t, mount(m::Actor::Root, TARGET)).unwrap();
    check(&mut work, &t);
    assert!(
        !work.solved(&t),
        "administrator is not the assigned ordinary mounter"
    );
}
#[test]
fn drafts_practice_records_resume_replay_and_forged_cache_invalidation_stay_separate() {
    let assignment = task(false);
    let mut example = assignment.clone();
    example.id = "different-workshop".into();
    // For this isolation test the Definition is not admitted as authored content;
    // independent input cases in real lesson packs must also use different UUIDs.
    let mut a = w::Work::new(&assignment);
    let mut b = w::Work::new(&example);
    let untouched = a.clone();
    save(&mut b, &example, "ro,noauto,user");
    b.apply(&example, mount(m::Actor::First, TARGET)).unwrap();
    check(&mut b, &example);
    assert!(b.solved(&example));
    assert_eq!(a, untouched);
    save(&mut a, &assignment, "ro,noauto,user");
    a.apply(&assignment, mount(m::Actor::First, TARGET))
        .unwrap();
    check(&mut a, &assignment);
    assert!(a.solved(&assignment));
    let raw = serde_json::to_string(&a).unwrap();
    assert!(!raw.contains("witness"));
    let mut restored: w::Work = serde_json::from_str(&raw).unwrap();
    restored.validate(&assignment).unwrap();
    assert!(!restored.solved(&assignment));
    assert!(restored.needs_verification(&assignment));
    check(&mut restored, &assignment);
    assert!(restored.solved(&assignment));
    let old = restored.result.clone().unwrap();
    restored.edit(config("ro,noauto,nouser")).unwrap();
    assert!(!restored.solved(&assignment));
    let mut forged = old;
    forged.draft = restored.draft.clone();
    assert!(!restored.store(forged, &assignment));
    let mut impossible: w::Work = serde_json::from_str(&raw).unwrap();
    impossible.history.insert(0, mount(m::Actor::First, TARGET));
    assert!(
        impossible.validate(&assignment).is_err(),
        "source starter is privileged-only; forged successful event must fail replay"
    );
}
#[test]
fn authored_seed_errors_import_limits_failed_save_and_stop_never_award_credit() {
    let t = task(false);
    let mut invalid = t.clone();
    invalid.seed.push(mount(m::Actor::First, TARGET));
    assert!(invalid.validate_authored().is_err());
    let mut invalid = t.clone();
    invalid.cases[3].seed.push(mount(m::Actor::First, TARGET));
    assert!(invalid.validate_authored().is_err());
    let mut work = w::Work::new(&t);
    let live = work.live(&t).unwrap();
    let invalid = "not a valid mount entry".to_owned();
    assert!(
        work.apply(
            &t,
            w::Action::Save {
                configuration: invalid.clone()
            }
        )
        .is_err()
    );
    assert_eq!(work.draft, invalid);
    assert_eq!(work.live(&t).unwrap(), live);
    assert!(work.history.is_empty());
    work.edit("x".repeat(16_385)).unwrap_err();
    assert_eq!(work.draft, invalid);
    let mut run = w::Run::new(&t, &work).unwrap();
    run.tick();
    run.stop();
    assert!(run.done());
    assert!(run.record().is_none());
    assert!(!work.solved(&t));
    let mut long = w::Work::new(&t);
    long.history = vec![
        w::Action::Save {
            configuration: t.starter.clone()
        };
        49
    ];
    assert!(long.validate(&t).is_err());
}
