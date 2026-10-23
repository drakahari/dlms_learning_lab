//! Educational account diagnostics only; controlled virtual data, no host tools.
use dlms_learning_lab::accounts::{self, EpochDay, Group, Kind, Requirement, State, User};
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn initial() -> State {
    let pack: Value =
        serde_json::from_str(include_str!("fixtures/accounts-workspace-preview.json")).unwrap();
    serde_json::from_value(pack["scenarios"][0]["initial"]["simulation"].clone()).unwrap()
}
fn wanted_user() -> User {
    User {
        uid: 1200,
        primary_gid: 1000,
        supplementary: BTreeSet::from([2001]),
        home: "/home/nola".into(),
        shell: "/bin/bash".into(),
        comment: String::new(),
        expires: None,
    }
}
fn with_user(initial: &State) -> State {
    changed(initial, |v| {
        v["configuration"]["users"]["nola"] = serde_json::to_value(wanted_user()).unwrap()
    })
}
fn changed(state: &State, f: impl FnOnce(&mut Value)) -> State {
    let mut value = serde_json::to_value(state).unwrap();
    f(&mut value);
    serde_json::from_value(value).unwrap()
}
fn check(initial: &State, state: &State, requirement: Requirement) -> (bool, String) {
    assert!(state.valid_against(initial));
    assert!(accounts::valid_requirements(
        initial,
        std::slice::from_ref(&requirement)
    ));
    let rows = accounts::checks(initial, state, &[requirement]);
    assert_eq!(rows.len(), 1);
    assert!(rows[0].1.len() <= accounts::MAX_ERROR_BYTES);
    rows.into_iter().next().unwrap()
}
fn assert_wrong(result: (bool, String), fragments: &[&str]) {
    assert!(!result.0);
    for fragment in fragments {
        assert!(
            result.1.contains(fragment),
            "missing {fragment:?}: {}",
            result.1
        );
    }
    assert!(
        !result.1.contains("useradd ") && !result.1.contains("usermod "),
        "Do not dump a worked command: {}",
        result.1
    );
}

#[test]
fn user_feedback_distinguishes_missing_identity_groups_home_shell_and_date_metadata() {
    let initial = initial();
    let desired = Requirement::User {
        name: "nola".into(),
        expected: wanted_user(),
    };
    assert_wrong(
        check(&initial, &initial, desired.clone()),
        &["nola", "record is missing", "getent passwd"],
    );
    let correct = with_user(&initial);
    let cases = [
        ("uid", json!(1201), vec!["UID is 1201", "requires 1200"]),
        (
            "primary_gid",
            json!(2000),
            vec!["primary GID is 2000", "requires 1000", "supplementary"],
        ),
        (
            "supplementary",
            json!([2000]),
            vec![
                "supplementary GIDs are [2000]",
                "requested [2001]",
                "Replacing",
                "appending",
            ],
        ),
        (
            "home",
            json!("/home/elsewhere"),
            vec![
                "home is /home/elsewhere",
                "requires /home/nola",
                "checked separately",
            ],
        ),
        (
            "shell",
            json!("/usr/sbin/nologin"),
            vec![
                "shell is /usr/sbin/nologin",
                "requires /bin/bash",
                "not a login test",
            ],
        ),
        (
            "expires",
            json!(EpochDay::parse("2026-11-30").unwrap()),
            vec![
                "expiry metadata",
                "2026-11-30",
                "requested not set",
                "no authentication",
            ],
        ),
        (
            "comment",
            json!("Changed comment"),
            vec!["account comment differs", "getent passwd"],
        ),
    ];
    for (field, value, fragments) in cases {
        let wrong = changed(&correct, |v| {
            v["configuration"]["users"]["nola"][field] = value
        });
        assert_wrong(check(&initial, &wrong, desired.clone()), &fragments);
    }
    let (passed, message) = check(&initial, &correct, desired);
    assert!(passed);
    assert_eq!(
        message,
        "nola: requested account identity, membership and policy"
    );
    let two_wrong = changed(&correct, |v| {
        v["configuration"]["users"]["nola"]["uid"] = 1201.into();
        v["configuration"]["users"]["nola"]["primary_gid"] = 2000.into();
    });
    assert_wrong(
        check(
            &initial,
            &two_wrong,
            Requirement::User {
                name: "nola".into(),
                expected: wanted_user(),
            },
        ),
        &["UID is 1201"],
    );
}

#[test]
fn groups_files_and_removal_feedback_name_the_specific_remaining_difference() {
    let initial = initial();
    let desired = Requirement::Group {
        name: "reports".into(),
        expected: Group { gid: 2500 },
    };
    assert_wrong(
        check(&initial, &initial, desired.clone()),
        &["group record is missing", "GID 2500"],
    );
    let wrong_group = changed(&initial, |v| {
        v["configuration"]["groups"]["reports"] = json!({"gid":2400})
    });
    assert_wrong(
        check(&initial, &wrong_group, desired),
        &["GID is 2400", "requires 2500"],
    );
    let path = "/home/mira/brief.txt";
    let expected = initial.configuration().entries[path].clone();
    for (field, value, fragments) in [
        (
            "uid",
            json!(1102),
            vec!["owner UID is 1102", "requires 1101", "ls -ln"],
        ),
        (
            "gid",
            json!(2000),
            vec!["group GID is 2000", "requires 1000"],
        ),
        ("mode", json!(0o600), vec!["mode is 0600", "requires 0640"]),
        (
            "data",
            json!({"kind":"directory"}),
            vec!["entry is a directory", "requires a regular file"],
        ),
    ] {
        let wrong = changed(&initial, |v| {
            v["configuration"]["entries"][path][field] = value
        });
        assert_wrong(
            check(
                &initial,
                &wrong,
                Requirement::Entry {
                    path: path.into(),
                    expected: expected.clone(),
                },
            ),
            &fragments,
        );
    }
    let missing = changed(&initial, |v| {
        v["configuration"]["entries"]
            .as_object_mut()
            .unwrap()
            .remove(path);
    });
    assert_wrong(
        check(
            &initial,
            &missing,
            Requirement::Entry {
                path: path.into(),
                expected: expected.clone(),
            },
        ),
        &[path, "entry is missing"],
    );
    let mut wanted_directory = expected.clone();
    wanted_directory.data = Kind::Directory;
    assert_wrong(
        check(
            &initial,
            &initial,
            Requirement::Entry {
                path: path.into(),
                expected: wanted_directory,
            },
        ),
        &["entry is a regular file", "requires a directory"],
    );
    assert_wrong(
        check(
            &initial,
            &initial,
            Requirement::UserAbsent {
                name: "mira".into(),
                uid: 1101,
            },
        ),
        &["record is still present", "UID 1101"],
    );
    assert_wrong(
        check(
            &initial,
            &initial,
            Requirement::GroupAbsent {
                name: "review".into(),
                gid: 2000,
            },
        ),
        &["group record is still present", "GID 2000"],
    );
    assert_wrong(
        check(
            &initial,
            &initial,
            Requirement::AbsentPath {
                path: "/home/mira".into(),
            },
        ),
        &["still present", "record alone does not remove its home"],
    );
    let renamed_user = changed(&initial, |v| {
        let record = v["configuration"]["users"]
            .as_object_mut()
            .unwrap()
            .remove("mira")
            .unwrap();
        v["configuration"]["users"]["other_name"] = record;
    });
    assert_wrong(
        check(
            &initial,
            &renamed_user,
            Requirement::UserAbsent {
                name: "mira".into(),
                uid: 1101,
            },
        ),
        &["name is absent", "UID 1101", "other_name"],
    );
    let renamed_group = changed(&initial, |v| {
        let record = v["configuration"]["groups"]
            .as_object_mut()
            .unwrap()
            .remove("review")
            .unwrap();
        v["configuration"]["groups"]["other_group"] = record;
    });
    assert_wrong(
        check(
            &initial,
            &renamed_group,
            Requirement::GroupAbsent {
                name: "review".into(),
                gid: 2000,
            },
        ),
        &["name is absent", "GID 2000", "other_group"],
    );
}

#[test]
fn whole_configuration_feedback_locates_unrequested_named_changes_without_dumping_contents() {
    let initial = initial();
    let expected = Requirement::Configuration {
        expected: initial.configuration().clone(),
    };
    for (field, wrong, fragments) in [
        (
            "user",
            changed(&initial, |v| {
                v["configuration"]["users"]["mira"]["shell"] = "/usr/sbin/nologin".into()
            }),
            vec!["First remaining difference", "mira", "shell"],
        ),
        (
            "extra user",
            with_user(&initial),
            vec!["nola", "unexpected account record", "UID 1200"],
        ),
        (
            "group",
            changed(&initial, |v| {
                v["configuration"]["groups"]["unexpected"] = json!({"gid":2500});
            }),
            vec!["unexpected", "group record", "GID 2500"],
        ),
        (
            "file",
            changed(&initial, |v| {
                v["configuration"]["entries"]["/archive/keep.txt"]["uid"] = 1102.into()
            }),
            vec!["/archive/keep.txt", "owner UID is 1102"],
        ),
        (
            "extra file",
            changed(&initial, |v| {
                v["configuration"]["entries"]["/home/mira/unasked.txt"] = json!({"uid":1101,"gid":1000,"mode":384,"data":{"kind":"text","contents":"Synthetic extra text\n"}});
            }),
            vec!["/home/mira/unasked.txt", "unexpected entry remains"],
        ),
    ] {
        let result = check(&initial, &wrong, expected.clone());
        assert!(!result.0, "{field}");
        assert_wrong(result, &fragments);
    }
    let path = "/home/mira/brief.txt";
    let payload = "SYNTHETIC_BIG_PAYLOAD_".to_owned() + &"🙂".repeat(16_000);
    let big_file = changed(&initial, |v| {
        v["configuration"]["entries"][path]["data"]["contents"] = payload.into()
    });
    let (_, message) = check(
        &initial,
        &big_file,
        Requirement::Entry {
            path: path.into(),
            expected: initial.configuration().entries[path].clone(),
        },
    );
    assert!(message.contains("text differs") && message.contains("bytes now"));
    assert!(!message.contains("SYNTHETIC_BIG_PAYLOAD_") && !message.contains('🙂'));
    assert!(message.len() <= accounts::MAX_ERROR_BYTES);
}

#[test]
fn expiry_range_diagnostics_are_stored_metadata_not_an_authentication_claim() {
    let initial = initial();
    let first = EpochDay::parse("2026-11-01").unwrap();
    let last = EpochDay::parse("2026-11-30").unwrap();
    let r = Requirement::ExpiryRange {
        name: "mira".into(),
        first,
        last,
    };
    assert_wrong(
        check(&initial, &initial, r.clone()),
        &[
            "expiry metadata is not set",
            "2026-11-01",
            "2026-11-30",
            "not a login",
        ],
    );
    let wrong = changed(&initial, |v| {
        v["configuration"]["users"]["mira"]["expires"] =
            json!(EpochDay::parse("2026-12-31").unwrap())
    });
    assert_wrong(
        check(&initial, &wrong, r),
        &["2026-12-31", "inclusive", "stored metadata only"],
    );
}

#[test]
fn feedback_keeps_the_previous_typed_booleans_and_invalid_context_gate() {
    let initial = initial();
    let current = with_user(&initial);
    let requirements = vec![
        Requirement::User {
            name: "nola".into(),
            expected: wanted_user(),
        },
        Requirement::Group {
            name: "staff".into(),
            expected: initial.configuration().groups["staff"].clone(),
        },
        Requirement::Group {
            name: "reports".into(),
            expected: Group { gid: 2500 },
        },
        Requirement::UserAbsent {
            name: "mira".into(),
            uid: 1101,
        },
        Requirement::GroupAbsent {
            name: "review".into(),
            gid: 2000,
        },
        Requirement::Entry {
            path: "/home/mira/brief.txt".into(),
            expected: initial.configuration().entries["/home/mira/brief.txt"].clone(),
        },
        Requirement::AbsentPath {
            path: "/home/mira/brief.txt".into(),
        },
        Requirement::Configuration {
            expected: current.configuration().clone(),
        },
        Requirement::ExpiryRange {
            name: "mira".into(),
            first: EpochDay::parse("2026-11-01").unwrap(),
            last: EpochDay::parse("2026-11-30").unwrap(),
        },
    ];
    assert!(accounts::valid_requirements(&initial, &requirements));
    let c = current.configuration();
    let before = requirements
        .iter()
        .map(|r| match r {
            Requirement::User { name, expected } => c.users.get(name) == Some(expected),
            Requirement::Group { name, expected } => c.groups.get(name) == Some(expected),
            Requirement::UserAbsent { name, uid } => {
                !c.users.contains_key(name) && !c.users.values().any(|u| u.uid == *uid)
            }
            Requirement::GroupAbsent { name, gid } => {
                !c.groups.contains_key(name) && !c.groups.values().any(|g| g.gid == *gid)
            }
            Requirement::Entry { path, expected } => c.entries.get(path) == Some(expected),
            Requirement::AbsentPath { path } => !c.entries.contains_key(path),
            Requirement::Configuration { expected } => c == expected,
            Requirement::ExpiryRange { name, first, last } => c
                .users
                .get(name)
                .and_then(|u| u.expires)
                .is_some_and(|d| d >= *first && d <= *last),
        })
        .collect::<Vec<_>>();
    let after = accounts::checks(&initial, &current, &requirements);
    assert_eq!(after.iter().map(|c| c.0).collect::<Vec<_>>(), before);
    assert!(after.iter().all(|c| c.1.len() <= accounts::MAX_ERROR_BYTES));
    let bad_context = changed(&initial, |v| {
        v["configuration"]["users"]["root"]["comment"] = "Changed protected record".into()
    });
    assert_eq!(
        accounts::checks(&initial, &bad_context, &requirements),
        vec![(
            false,
            "Account fixture, identity or outcome requirements do not match".into()
        )]
    );
    let invalid = Requirement::User {
        name: "é".repeat(1000),
        expected: wanted_user(),
    };
    assert!(!accounts::valid_requirements(
        &initial,
        std::slice::from_ref(&invalid)
    ));
    assert_eq!(
        accounts::checks(&initial, &current, &[invalid]),
        vec![(
            false,
            "Account fixture, identity or outcome requirements do not match".into()
        )]
    );
}

#[test]
fn incomplete_known_forms_and_genuinely_unsupported_commands_differ_without_mutation() {
    use dlms_learning_lab::{engine, pack};
    let scenario = pack::load(include_str!("fixtures/accounts-workspace-preview.json"))
        .unwrap()
        .remove(0);
    for (command, required) in [
        ("usermod -G", "GROUP_LIST and USER"),
        ("usermod -G audit", "GROUP_LIST and USER"),
        ("usermod --groups", "GROUP_LIST and USER"),
        ("usermod -aG", "GROUP_LIST and USER"),
        ("usermod -aG audit", "GROUP_LIST and USER"),
        ("usermod -a -G", "GROUP_LIST and USER"),
        ("usermod --append --groups audit", "GROUP_LIST and USER"),
        ("usermod -G audit -a", "GROUP_LIST and USER"),
        ("usermod -g", "GROUP and USER"),
        ("usermod --gid review", "GROUP and USER"),
        ("usermod -e", "DATE and USER"),
        ("usermod --expiredate 2026-11-30", "DATE and USER"),
    ] {
        let error = accounts::parse(command).unwrap_err();
        assert_eq!(error.kind, accounts::ErrorKind::Invalid);
        assert!(
            error.message.contains(required),
            "{command}: {}",
            error.message
        );
        assert!(!error.message.contains("bounded account help"));
        let mut state = scenario.initial.clone();
        let before = state.clone();
        let output = engine::execute_for_goal(&mut state, &scenario.goal, command).unwrap_err();
        assert!(output.contains(required));
        assert_eq!(state, before);
    }
    for command in [
        "usermod -L mira",
        "usermod -G audit -Z mira",
        "passwd mira",
        "id mira | cat",
    ] {
        let error = accounts::parse(command).unwrap_err();
        assert_eq!(error.kind, accounts::ErrorKind::Unsupported);
        assert!(error.message.contains("not supported") && error.message.contains("Open Help"));
        let mut state = scenario.initial.clone();
        let before = state.clone();
        assert!(engine::execute_for_goal(&mut state, &scenario.goal, command).is_err());
        assert_eq!(state, before);
    }
    for command in [
        "usermod -G audit mira",
        "usermod -aG review mira",
        "usermod -a -G review mira",
        "usermod -G review -a mira",
        "usermod --gid review mira",
        "usermod -e '' mira",
        "usermod --expiredate 2026-11-30 mira",
    ] {
        assert!(
            accounts::parse(command).is_ok(),
            "Previously supported form changed: {command}"
        );
    }
}
