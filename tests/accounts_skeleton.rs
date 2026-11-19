//! Known-leaf template regression tests. No host account or file operations.
use dlms_learning_lab::{
    accounts::{self, Action, Entry, ErrorKind, Kind, Requirement, SkeletonLeaf, State},
    accounts_teaching,
};
use std::collections::BTreeMap;
fn old() -> State {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/accounts-lifecycle-v1.json")).unwrap();
    serde_json::from_value(v["scenarios"][0]["initial"]["simulation"].clone()).unwrap()
}
fn entry(mode: u16, data: Kind) -> Entry {
    Entry {
        uid: 0,
        gid: 0,
        mode,
        data,
    }
}
fn text(s: &str) -> Kind {
    Kind::Text { contents: s.into() }
}
fn fixture() -> State {
    let old = old();
    let mut p = old.policy().clone();
    let mut c = old.configuration().clone();
    p.fixture = "skeleton-core-capability".into();
    c.entries
        .insert("/etc/skel/.config".into(), entry(0o750, Kind::Directory));
    c.entries.insert(
        "/etc/skel/.config/client".into(),
        entry(0o750, Kind::Directory),
    );
    c.entries.insert(
        "/etc/skel/.config/client/settings.conf".into(),
        entry(0o600, text("region=old\n")),
    );
    c.entries.insert(
        "/etc/skel/private-note.txt".into(),
        entry(
            0o600,
            text("SYNTHETIC private note; never seed a new account\n"),
        ),
    );
    let pax = &c.users["pax"];
    let home = pax.home.clone();
    let uid = pax.uid;
    let gid = pax.primary_gid;
    c.entries.entry(home.clone()).or_insert(Entry {
        uid,
        gid,
        mode: 0o700,
        data: Kind::Directory,
    });
    c.entries.insert(
        format!("{home}/.profile"),
        Entry {
            uid,
            gid,
            mode: 0o600,
            data: text("Existing home: keep its own preferences\n"),
        },
    );
    p.editable_skeleton_leaves = BTreeMap::from([
        (
            "/etc/skel/.profile".into(),
            SkeletonLeaf {
                uid: 0,
                gid: 0,
                allow_remove: false,
            },
        ),
        (
            "/etc/skel/.config/client/settings.conf".into(),
            SkeletonLeaf {
                uid: 0,
                gid: 0,
                allow_remove: false,
            },
        ),
        (
            "/etc/skel/private-note.txt".into(),
            SkeletonLeaf {
                uid: 0,
                gid: 0,
                allow_remove: true,
            },
        ),
    ]);
    let state = State::new(p, c).unwrap();
    assert!(state.valid_initial());
    state
}
fn save(current: &mut State, initial: &State, path: &str, contents: &str, mode: u16) {
    accounts_teaching::apply_in_context(
        initial,
        current,
        &Action::SaveSkeletonLeaf {
            path: path.into(),
            contents: contents.into(),
            mode,
        },
    )
    .unwrap();
}
fn edited() -> (State, State) {
    let initial = fixture();
    let mut state = initial.clone();
    save(
        &mut state,
        &initial,
        "/etc/skel/.profile",
        "TRAINING_REGION=west\nexport TRAINING_REGION\n",
        0o644,
    );
    save(
        &mut state,
        &initial,
        "/etc/skel/.config/client/settings.conf",
        "region=west\n",
        0o640,
    );
    accounts_teaching::apply_in_context(
        &initial,
        &mut state,
        &Action::RemoveSkeletonLeaf {
            path: "/etc/skel/private-note.txt".into(),
        },
    )
    .unwrap();
    (initial, state)
}
#[test]
fn old_empty_option_wire_and_frozen_template_behavior_remain_unchanged() {
    let initial = old();
    assert!(initial.valid_initial());
    assert!(initial.policy().editable_skeleton_leaves.is_empty());
    let v = serde_json::to_value(&initial).unwrap();
    assert!(v["policy"].get("editable_skeleton_leaves").is_none());
    let decoded = State::from_json(&serde_json::to_string(&initial).unwrap()).unwrap();
    assert_eq!(decoded, initial);
    let mut current = initial.clone();
    let before = current.clone();
    let error = current
        .apply(&Action::SaveSkeletonLeaf {
            path: "/etc/skel/.profile".into(),
            contents: "changed".into(),
            mode: 0o644,
        })
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Protected);
    assert_eq!(current, before);
    assert!(!accounts::valid_requirements(
        &initial,
        &[Requirement::AbsentPath {
            path: "/etc/skel/.profile".into()
        }]
    ));
}
#[test]
fn opt_in_save_changes_only_known_leaf_text_mode_not_identity_or_other_entries() {
    let initial = fixture();
    let mut state = initial.clone();
    save(
        &mut state,
        &initial,
        "/etc/skel/.profile",
        "New default\n",
        0o640,
    );
    assert!(state.valid_against(&initial));
    assert_eq!(state.policy(), initial.policy());
    let entry = &state.configuration().entries["/etc/skel/.profile"];
    assert_eq!((entry.uid, entry.gid, entry.mode), (0, 0, 0o640));
    assert_eq!(entry.data, text("New default\n"));
    for (path, entry) in &initial.configuration().entries {
        if path != "/etc/skel/.profile" {
            assert_eq!(state.configuration().entries.get(path), Some(entry));
        }
    }
    assert_eq!(state.configuration().users, initial.configuration().users);
    assert_eq!(state.configuration().groups, initial.configuration().groups);
}
#[test]
fn removal_and_recreation_retain_identity_after_the_previous_entry_has_disappeared() {
    let initial = fixture();
    let mut state = initial.clone();
    let path = "/etc/skel/private-note.txt";
    accounts_teaching::apply_in_context(
        &initial,
        &mut state,
        &Action::RemoveSkeletonLeaf { path: path.into() },
    )
    .unwrap();
    assert!(state.valid());
    assert!(!state.valid_initial());
    assert!(state.valid_against(&initial));
    assert!(!state.configuration().entries.contains_key(path));
    let restored = State::from_json(&serde_json::to_string(&state).unwrap()).unwrap();
    assert_eq!(restored, state);
    assert!(restored.valid_against(&initial));
    save(
        &mut state,
        &initial,
        path,
        "Replacement public default\n",
        0o644,
    );
    let now = &state.configuration().entries[path];
    assert_eq!((now.uid, now.gid), (0, 0));
    assert!(matches!(now.data, Kind::Text { .. }));
    assert!(state.valid_initial());
}
#[test]
fn actual_useradd_default_and_explicit_skeleton_copy_modified_dot_and_nested_defaults() {
    for flag in ["-m", "-m -k /etc/skel"] {
        let (initial, mut state) = edited();
        let before = state.configuration().clone();
        accounts_teaching::execute_in_context(
            &initial,
            &mut state,
            &format!("useradd -N -u 3201 -g staff -d /home/probe -s /bin/bash {flag} probe"),
        )
        .unwrap();
        let c = state.configuration();
        assert_eq!(c.entries["/home/probe"].mode, 0o700);
        for suffix in [
            "/.profile",
            "/.config",
            "/.config/client",
            "/.config/client/settings.conf",
        ] {
            let src = &before.entries[&format!("/etc/skel{suffix}")];
            let dst = &c.entries[&format!("/home/probe{suffix}")];
            assert_eq!(dst.data, src.data);
            assert_eq!(dst.mode, src.mode);
            assert_eq!((dst.uid, dst.gid), (3201, 1000));
        }
        assert!(!c.entries.contains_key("/home/probe/private-note.txt"));
        for (path, entry) in &before.entries {
            assert_eq!(
                c.entries.get(path),
                Some(entry),
                "copy does not update original/old homes: {path}"
            );
        }
        assert!(state.valid_against(&initial));
    }
}
#[test]
fn creating_before_maintenance_does_not_retroactively_update_the_existing_home() {
    let initial = fixture();
    let mut state = initial.clone();
    accounts_teaching::execute_in_context(
        &initial,
        &mut state,
        "useradd -N -u 3201 -g staff -d /home/probe -s /bin/bash -m probe",
    )
    .unwrap();
    let old_copy = state.configuration().entries["/home/probe/.profile"].clone();
    save(
        &mut state,
        &initial,
        "/etc/skel/.profile",
        "Different future default\n",
        0o640,
    );
    assert_eq!(
        state.configuration().entries["/home/probe/.profile"],
        old_copy
    );
    let wanted = Requirement::Entry {
        path: "/home/probe/.profile".into(),
        expected: Entry {
            uid: 3201,
            gid: 1000,
            mode: 0o640,
            data: text("Different future default\n"),
        },
    };
    assert!(accounts::valid_requirements(
        &initial,
        std::slice::from_ref(&wanted)
    ));
    assert!(!accounts::checks(&initial, &state, &[wanted])[0].0);
}
#[test]
fn metadata_only_creation_and_unrelated_unchanged_skeleton_do_not_fake_population() {
    let (initial, mut no_home) = edited();
    accounts_teaching::execute_in_context(
        &initial,
        &mut no_home,
        "useradd -N -u 3201 -g staff -d /home/probe -s /bin/bash -M probe",
    )
    .unwrap();
    assert!(!no_home.configuration().entries.contains_key("/home/probe"));
    let (initial, mut other) = edited();
    accounts_teaching::execute_in_context(
        &initial,
        &mut other,
        "useradd -N -u 3201 -g staff -d /home/probe -s /bin/bash -m -k /etc/skel-field probe",
    )
    .unwrap();
    assert_eq!(
        other.configuration().entries["/home/probe/.profile"].data,
        initial.configuration().entries["/etc/skel-field/.profile"].data
    );
    assert_ne!(
        other.configuration().entries["/home/probe/.profile"].data,
        other.configuration().entries["/etc/skel/.profile"].data
    );
}
#[test]
fn directories_unlisted_templates_other_home_and_nonremovable_leaves_refuse_atomically() {
    let initial = fixture();
    for path in [
        "/etc/skel",
        "/etc/skel/.config",
        "/etc/skel-field/.profile",
        "/etc/new-file",
        "/home/pax/.profile",
    ] {
        let mut state = initial.clone();
        assert!(
            state
                .apply(&Action::SaveSkeletonLeaf {
                    path: path.into(),
                    contents: "bad".into(),
                    mode: 0o600
                })
                .is_err()
        );
        assert_eq!(state, initial);
        assert!(
            state
                .apply(&Action::RemoveSkeletonLeaf { path: path.into() })
                .is_err()
        );
        assert_eq!(state, initial);
    }
    let mut state = initial.clone();
    assert!(
        state
            .apply(&Action::RemoveSkeletonLeaf {
                path: "/etc/skel/.profile".into()
            })
            .is_err()
    );
    assert_eq!(state, initial);
}
#[test]
fn oversized_nul_special_mode_or_alias_paths_refuse_without_publishing_a_candidate() {
    let initial = fixture();
    for action in [
        Action::SaveSkeletonLeaf {
            path: "/etc/skel/.profile".into(),
            contents: "x".repeat(accounts::MAX_FILE_BYTES + 1),
            mode: 0o644,
        },
        Action::SaveSkeletonLeaf {
            path: "/etc/skel/.profile".into(),
            contents: "x\0y".into(),
            mode: 0o644,
        },
        Action::SaveSkeletonLeaf {
            path: "/etc/skel/.profile".into(),
            contents: "x".into(),
            mode: 0o1644,
        },
        Action::SaveSkeletonLeaf {
            path: "/etc/skel/../skel/.profile".into(),
            contents: "x".into(),
            mode: 0o644,
        },
    ] {
        let mut state = initial.clone();
        assert!(state.apply(&action).is_err());
        assert_eq!(state, initial);
    }
}
#[test]
fn declaration_rejects_directories_hard_protection_wrong_identity_external_and_excess_keys() {
    let initial = fixture();
    for path in ["/etc/skel", "/etc/skel/.config", "/home/pax/.profile"] {
        let mut p = initial.policy().clone();
        p.editable_skeleton_leaves.clear();
        p.editable_skeleton_leaves.insert(
            path.into(),
            SkeletonLeaf {
                uid: 0,
                gid: 0,
                allow_remove: true,
            },
        );
        assert!(State::new(p, initial.configuration().clone()).is_err());
    }
    let mut p = initial.policy().clone();
    p.protected_entries.insert("/etc/skel/.profile".into());
    assert!(State::new(p, initial.configuration().clone()).is_err());
    let mut p = initial.policy().clone();
    p.editable_skeleton_leaves
        .get_mut("/etc/skel/.profile")
        .unwrap()
        .uid = 1000;
    assert!(State::new(p, initial.configuration().clone()).is_err());
    let mut p = initial.policy().clone();
    for n in 0..5 {
        p.editable_skeleton_leaves.insert(
            format!("/etc/skel/new{n}"),
            SkeletonLeaf {
                uid: 0,
                gid: 0,
                allow_remove: true,
            },
        );
    }
    assert!(State::new(p, initial.configuration().clone()).is_err());
}
#[test]
fn missing_removable_leaf_can_be_runtime_state_but_never_a_held_initial_declaration() {
    let initial = fixture();
    let mut c = initial.configuration().clone();
    c.entries.remove("/etc/skel/private-note.txt");
    let runtime = State::new(initial.policy().clone(), c).unwrap();
    assert!(runtime.valid());
    assert!(!runtime.valid_initial());
    assert!(runtime.valid_against(&initial));
    assert!(!runtime.valid_against(&runtime));
    assert!(!accounts::valid_requirements(
        &runtime,
        &[Requirement::AbsentPath {
            path: "/etc/skel/private-note.txt".into()
        }]
    ));
}
#[test]
fn changed_imported_policy_cannot_authorize_itself_or_replace_a_current_context() {
    let initial = fixture();
    let mut p = initial.policy().clone();
    p.editable_skeleton_leaves
        .get_mut("/etc/skel/.profile")
        .unwrap()
        .allow_remove = true;
    let forged = State::new(p, initial.configuration().clone()).unwrap();
    assert!(forged.valid_initial());
    assert!(!forged.valid_against(&initial));
    let mut current = forged.clone();
    assert!(
        accounts_teaching::apply_in_context(
            &initial,
            &mut current,
            &Action::RemoveSkeletonLeaf {
                path: "/etc/skel/.profile".into()
            }
        )
        .is_err()
    );
    assert_eq!(current, forged);
}
#[test]
fn after_removal_imported_uid_gid_or_directory_kind_cannot_recreate_another_identity() {
    let (initial, mut state) = edited();
    let path = "/etc/skel/private-note.txt";
    for e in [
        Entry {
            uid: 1000,
            gid: 0,
            mode: 0o644,
            data: text("bad"),
        },
        Entry {
            uid: 0,
            gid: 1000,
            mode: 0o644,
            data: text("bad"),
        },
        entry(0o700, Kind::Directory),
    ] {
        let mut c = state.configuration().clone();
        c.entries.insert(path.into(), e);
        assert!(State::new(initial.policy().clone(), c).is_err());
    }
    save(&mut state, &initial, path, "Allowed recreation\n", 0o640);
    assert_eq!(state.configuration().entries[path].uid, 0);
    assert_eq!(state.configuration().entries[path].gid, 0);
    let before = state.clone();
    assert!(
        state
            .apply(&Action::Own {
                path: path.into(),
                user: "pax".into(),
                group: "staff".into()
            })
            .is_err()
    );
    assert_eq!(state, before);
}
#[test]
fn goals_allow_only_opted_leaf_content_and_permitted_absence_with_full_preservation() {
    let (initial, state) = edited();
    let desired = Requirement::Configuration {
        expected: state.configuration().clone(),
    };
    assert!(accounts::valid_requirements(
        &initial,
        std::slice::from_ref(&desired)
    ));
    assert!(accounts::checks(&initial, &state, &[desired])[0].0);
    let allowed = Requirement::Entry {
        path: "/etc/skel/.profile".into(),
        expected: state.configuration().entries["/etc/skel/.profile"].clone(),
    };
    assert!(accounts::valid_requirements(&initial, &[allowed]));
    assert!(accounts::valid_requirements(
        &initial,
        &[Requirement::AbsentPath {
            path: "/etc/skel/private-note.txt".into()
        }]
    ));
    assert!(!accounts::valid_requirements(
        &initial,
        &[Requirement::AbsentPath {
            path: "/etc/skel/.profile".into()
        }]
    ));
    for path in ["/etc/skel-field/.profile", "/etc/skel/.config"] {
        let e = Requirement::Entry {
            path: path.into(),
            expected: entry(0o600, text("not allowed")),
        };
        assert!(!accounts::valid_requirements(&initial, &[e]));
    }
}
#[test]
fn native_actions_have_no_fake_linux_terminal_spelling_or_host_dispatch() {
    let initial = fixture();
    for input in [
        "save-skeleton /etc/skel/.profile x",
        "cp /work/profile /etc/skel/.profile",
        "chmod 644 /etc/skel/.profile",
        "sh -c id",
        "sudo useradd probe",
    ] {
        let mut state = initial.clone();
        assert!(accounts_teaching::execute_in_context(&initial, &mut state, input).is_err());
        assert_eq!(state, initial);
    }
}
#[test]
fn new_policy_json_bounds_and_unknown_identity_fields_are_not_silently_accepted() {
    let initial = fixture();
    let raw = serde_json::to_string(&initial).unwrap();
    assert_eq!(State::from_json(&raw).unwrap(), initial);
    let bad = raw.replace(
        "\"uid\":0,\"gid\":0,\"allow_remove\":false",
        "\"uid\":0,\"gid\":0,\"allow_remove\":false,\"kind\":\"directory\"",
    );
    assert_ne!(bad, raw);
    assert!(State::from_json(&bad).is_err());
    let mut v = serde_json::to_value(&initial).unwrap();
    v["policy"]["editable_skeleton_leaves"]["/etc/skel/.profile"]["allow_remove"] =
        serde_json::json!(1);
    assert!(State::from_json(&serde_json::to_string(&v).unwrap()).is_err());
    let mut v = serde_json::to_value(&initial).unwrap();
    v["policy"]["editable_skeleton_leaves"]["/etc/skel/.profile"]["uid"] = serde_json::json!(60001);
    assert!(State::from_json(&serde_json::to_string(&v).unwrap()).is_err());
}
#[test]
fn aggregate_text_limit_refuses_before_publication_even_when_one_leaf_is_in_range() {
    let initial = fixture();
    let mut c = initial.configuration().clone();
    let used: usize = c
        .entries
        .values()
        .map(|e| match &e.data {
            Kind::Text { contents } => contents.len(),
            Kind::Directory => 0,
        })
        .sum();
    let mut remaining = accounts::MAX_TEXT_BYTES - used;
    let mut n = 0;
    while remaining > 0 {
        let amount = remaining.min(accounts::MAX_FILE_BYTES);
        c.entries.insert(
            format!("/srv/filler-{n}"),
            entry(0o644, text(&"x".repeat(amount))),
        );
        remaining -= amount;
        n += 1;
    }
    let full = State::new(initial.policy().clone(), c).unwrap();
    assert!(full.valid_initial());
    let mut state = full.clone();
    assert!(
        state
            .apply(&Action::SaveSkeletonLeaf {
                path: "/etc/skel/.profile".into(),
                contents: "x".repeat(accounts::MAX_FILE_BYTES),
                mode: 0o644
            })
            .is_err()
    );
    assert_eq!(state, full);
}
#[test]
fn unsupported_useradd_skeleton_flags_and_home_adoption_remain_atomic() {
    let initial = fixture();
    for command in [
        "useradd -N -u 3201 -g staff -d /home/probe -s /bin/bash -M -k /etc/skel probe",
        "useradd -N -u 3201 -g staff -d /home/probe -s /bin/bash -m -k /etc/unknown probe",
        "useradd -N -u 3201 -g staff -d /home/pax -s /bin/bash -m probe",
        "useradd --copy-skeleton /etc/skel probe",
    ] {
        let mut state = initial.clone();
        assert!(accounts_teaching::execute_in_context(&initial, &mut state, command).is_err());
        assert_eq!(state, initial);
    }
}

#[test]
fn standalone_policy_reader_refuses_duplicate_excess_and_oversized_leaf_keys() {
    let initial = fixture();
    let raw = serde_json::to_string(initial.policy()).unwrap();
    let duplicate=raw.replace("\"editable_skeleton_leaves\":{", "\"editable_skeleton_leaves\":{\"/etc/skel/.profile\":{\"uid\":0,\"gid\":0,\"allow_remove\":false},");
    assert_ne!(duplicate, raw);
    assert!(serde_json::from_str::<accounts::Policy>(&duplicate).is_err());
    let mut v = serde_json::to_value(initial.policy()).unwrap();
    let leaves = v["editable_skeleton_leaves"].as_object_mut().unwrap();
    leaves.clear();
    for n in 0..5 {
        leaves.insert(
            format!("/etc/skel/leaf{n}"),
            serde_json::json!({"uid":0,"gid":0,"allow_remove":true}),
        );
    }
    assert!(serde_json::from_value::<accounts::Policy>(v.clone()).is_err());
    let leaves = v["editable_skeleton_leaves"].as_object_mut().unwrap();
    leaves.clear();
    leaves.insert(
        "x".repeat(129),
        serde_json::json!({"uid":0,"gid":0,"allow_remove":true}),
    );
    assert!(serde_json::from_value::<accounts::Policy>(v).is_err());
}
