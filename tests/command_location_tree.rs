//! Unexecuted private source proposal: no Goal/pack/UI glue is assumed here.
use dlms_learning_lab::engine::{self, Goal, State};
use dlms_learning_lab::{
    command_location::{Context, Mode, Query},
    engine::Node,
    filesystem::{
        Requirement, Tree,
        command_location_policy::{self as policy, Definition, Selection},
        path_lookup::{self, Profile},
    },
};
use std::collections::BTreeMap;
fn node(directory: bool, mode: u16, owner: &str, group: &str) -> Node {
    Node {
        directory,
        mode,
        owner: owner.into(),
        group: group.into(),
    }
}
fn fixture() -> (Context, Tree) {
    let mut nodes = BTreeMap::new();
    for p in [
        "/",
        "/home",
        "/home/alex",
        "/work",
        "/usr",
        "/usr/local",
        "/usr/local/bin",
        "/usr/bin",
        "/search",
        "/denied",
        "/targetdir",
        "/targetdir/report",
    ] {
        nodes.insert(p.into(), node(true, 0o555, "root", "root"));
    }
    nodes.get_mut("/home/alex").unwrap().mode = 0o700;
    nodes.get_mut("/home/alex").unwrap().owner = "alex".into();
    nodes.get_mut("/home/alex").unwrap().group = "staff".into();
    nodes.get_mut("/work").unwrap().owner = "alex".into();
    nodes.get_mut("/work").unwrap().group = "staff".into();
    nodes.get_mut("/work").unwrap().mode = 0o755;
    nodes.get_mut("/search").unwrap().mode = 0o111;
    nodes.get_mut("/denied").unwrap().mode = 0o700;
    let mut files = BTreeMap::new();
    for (p, mode, owner, group, body) in [
        (
            "/usr/local/bin/report",
            0o644,
            "alex",
            "staff",
            "preferred virtual artifact\n",
        ),
        (
            "/usr/bin/report",
            0o555,
            "root",
            "root",
            "fallback virtual artifact\n",
        ),
        (
            "/usr/bin/audit",
            0o555,
            "root",
            "root",
            "sibling virtual artifact\n",
        ),
        (
            "/search/report",
            0o111,
            "alex",
            "staff",
            "search-only artifact\n",
        ),
        (
            "/denied/report",
            0o555,
            "alex",
            "staff",
            "denied ancestor artifact\n",
        ),
    ] {
        nodes.insert(p.into(), node(false, mode, owner, group));
        files.insert(p.into(), body.into());
    }
    let tree = Tree {
        nodes,
        files,
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into(), "ops".into()]),
            ("blair".into(), vec!["staff".into()]),
        ]),
        user: "alex".into(),
        home: "/home/alex".into(),
        cwd: "/work".into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: None,
    };
    let c = Context {
        version: 1,
        source_id: "command-location-source".into(),
        path: "/usr/local/bin:/usr/bin".into(),
        names: vec!["report".into(), "audit".into(), "missing".into()],
    };
    assert!(tree.valid());
    c.validate_for(&tree).unwrap();
    (c, tree)
}

fn initial() -> Tree {
    let (mut c, mut tree) = fixture();
    c.names = vec!["report".into(), "audit".into()];
    tree.nodes
        .insert("/work/note".into(), node(false, 0o644, "alex", "staff"));
    tree.files
        .insert("/work/note".into(), "Keep this supplied note\n".into());
    tree.command_location = Some(c);
    tree
}
fn definition() -> Definition {
    let source = initial();
    let mut expected = source.clone();
    expected
        .nodes
        .get_mut("/usr/local/bin/report")
        .unwrap()
        .mode = 0o744;
    let d = Definition {
        version: 1,
        initial: Box::new(source),
        expected: Box::new(expected),
        selected: vec![
            Selection {
                name: "report".into(),
                path: "/usr/local/bin/report".into(),
            },
            Selection {
                name: "audit".into(),
                path: "/usr/bin/audit".into(),
            },
        ],
    };
    d.validate().unwrap();
    d
}
fn requirement(d: Definition) -> Requirement {
    Requirement::CommandLocation {
        definition: Box::new(d),
    }
}

#[test]
fn engine_requires_unique_authored_reference_for_commands_help_and_grading() {
    let d = definition();
    let original = State::Filesystem {
        tree: (*d.initial).clone(),
    };
    for requirements in [
        vec![],
        vec![Requirement::Mode {
            path: "/usr/local/bin/report".into(),
            mode: 0o644,
        }],
        vec![requirement(d.clone()), requirement(d.clone())],
    ] {
        let goal = Goal::Filesystem { requirements };
        let mut current = original.clone();
        assert!(engine::checks(&current, &goal).iter().any(|r| !r.0));
        assert!(!dlms_learning_lab::workspace::issues(&current, &goal).is_empty());
        for command in [
            "help",
            "show",
            "which report",
            "chmod u+x /usr/local/bin/report",
        ] {
            assert!(engine::execute_for_goal(&mut current, &goal, command).is_err());
            assert_eq!(current, original);
        }
    }
    let goal = Goal::Filesystem {
        requirements: vec![requirement(d.clone())],
    };
    let mut foreign = State::Script {
        source: "# Preserve this independent script draft\n".into(),
        run: None,
    };
    let before = foreign.clone();
    assert!(engine::execute_for_goal(&mut foreign, &goal, "printf changed").is_err());
    assert_eq!(
        foreign, before,
        "new filesystem source cannot authorize another family"
    );
    let mut erased = (*d.initial).clone();
    erased.command_location = None;
    let mut current = State::Filesystem { tree: erased };
    assert!(!engine::solved(&current, &goal));
    assert!(!dlms_learning_lab::workspace::issues(&current, &goal).is_empty());
    assert!(engine::execute_for_goal(&mut current, &goal, "help").is_err());
    let mut current = original.clone();
    for command in ["help", "show", "which report"] {
        assert!(engine::execute(&mut current, command).is_err());
        assert_eq!(current, original);
    }
}

#[test]
fn engine_actual_queries_and_mode_repair_use_current_state_and_clear_success_when_broken() {
    let d = definition();
    let mut current = State::Filesystem {
        tree: (*d.initial).clone(),
    };
    let goal = Goal::Filesystem {
        requirements: vec![requirement(d)],
    };
    assert!(!engine::solved(&current, &goal));
    assert_eq!(
        engine::execute_for_goal(&mut current, &goal, "which report").unwrap(),
        "/usr/bin/report\n"
    );
    engine::execute_for_goal(&mut current, &goal, "chmod u+x /usr/local/bin/report").unwrap();
    assert_eq!(
        engine::execute_for_goal(&mut current, &goal, "which report").unwrap(),
        "/usr/local/bin/report\n"
    );
    assert!(engine::solved(&current, &goal));
    assert!(dlms_learning_lab::workspace::issues(&current, &goal).is_empty());
    engine::execute_for_goal(&mut current, &goal, "chmod 0755 /usr/local/bin/report").unwrap();
    assert!(
        !engine::solved(&current, &goal),
        "correct provider with overbroad bits must fail"
    );
    engine::execute_for_goal(&mut current, &goal, "chmod 0744 /usr/local/bin/report").unwrap();
    assert!(engine::solved(&current, &goal));
    engine::execute_for_goal(&mut current, &goal, "chmod u-x /usr/local/bin/report").unwrap();
    assert!(!engine::solved(&current, &goal));
    assert!(
        dlms_learning_lab::workspace::issues(&current, &goal)
            .iter()
            .any(|m| m.contains("currently selects"))
    );
}
#[test]
fn old_none_json_and_command_results_stay_exact() {
    let (_, source) = fixture();
    assert!(source.command_location.is_none());
    let raw = serde_json::to_string(&source).unwrap();
    assert!(!raw.contains("command_location"));
    let restored: Tree = serde_json::from_str(&raw).unwrap();
    assert_eq!(restored, source);
    let mut old = source.clone();
    let a = old.execute("chmod u+x /usr/local/bin/report").unwrap();
    let mut through = source.clone();
    let b = through
        .execute_in_context(&source, "chmod u+x /usr/local/bin/report")
        .unwrap();
    assert_eq!(a, b);
    assert_eq!(old, through);
}
#[test]
fn source_admission_is_exact_but_live_check_accepts_actual_mode_repair() {
    let d = definition();
    let r = requirement(d.clone());
    assert!(r.valid());
    assert!(r.valid_for(&d.initial));
    assert!(
        !r.valid_for(&d.expected),
        "valid_for is original-source admission, not live grading"
    );
    assert!(!r.check(&d.initial).0);
    let mut current = (*d.initial).clone();
    current
        .execute_in_context(&d.initial, "chmod u+x /usr/local/bin/report")
        .unwrap();
    assert!(r.check(&current).0);
    assert!(current.clock > 0);
    assert_ne!(current.modified, d.expected.modified);
    assert_eq!(current.files, d.initial.files);
}
#[test]
fn current_query_output_tracks_permissions_and_source_free_execution_refuses() {
    let d = definition();
    let mut current = (*d.initial).clone();
    for command in [
        "which report",
        "chmod u+x /usr/local/bin/report",
        "help",
        "show",
    ] {
        let before = current.clone();
        assert!(current.execute(command).is_err());
        assert_eq!(current, before);
    }
    assert_eq!(
        current
            .execute_in_context(&d.initial, "which report")
            .unwrap(),
        "/usr/bin/report\n"
    );
    current
        .execute_in_context(&d.initial, "chmod 0744 /usr/local/bin/report")
        .unwrap();
    assert_eq!(
        current
            .execute_in_context(&d.initial, "which report")
            .unwrap(),
        "/usr/local/bin/report\n"
    );
    assert_eq!(
        current
            .execute_in_context(&d.initial, "type -a -p report")
            .unwrap(),
        "/usr/local/bin/report\n/usr/bin/report\n"
    );
}
#[test]
fn equivalent_supported_modes_pass_but_extra_permission_bits_do_not() {
    let d = definition();
    for command in [
        "chmod u+x /usr/local/bin/report",
        "chmod 0744 /usr/local/bin/report",
        "chmod u=rwx,g=r,o=r /usr/local/bin/report",
    ] {
        let mut current = (*d.initial).clone();
        current.execute_in_context(&d.initial, command).unwrap();
        assert!(d.check(&current).0, "{command}");
    }
    let mut current = (*d.expected).clone();
    current
        .execute_in_context(&d.initial, "chmod 0755 /usr/local/bin/report")
        .unwrap();
    assert!(!d.check(&current).0);
    assert_eq!(
        current
            .execute_in_context(&d.initial, "which report")
            .unwrap(),
        "/usr/local/bin/report\n",
        "selection alone is insufficient when preserved bits are wrong"
    );
}
#[test]
fn pwd_cd_previous_clock_and_touch_timestamps_are_ignored_only_when_valid() {
    let d = definition();
    let mut current = (*d.expected).clone();
    current
        .execute_in_context(&d.initial, "cd /usr/bin")
        .unwrap();
    assert_eq!(current.cwd, "/usr/bin");
    assert!(current.previous.is_some());
    current
        .execute_in_context(&d.initial, "touch /work/note")
        .unwrap();
    assert!(d.check(&current).0);
    current.clock = 1_000_001;
    assert!(!d.check(&current).0);
}
#[test]
fn ordinary_commands_cannot_publish_files_dirs_identity_or_mask_changes() {
    let d = definition();
    for command in [
        "mkdir /work/extra",
        "cp /work/note /work/copy",
        "mv /work/note /work/renamed",
        "rm /work/note",
        "chmod 0700 /work",
        "chmod 0600 /work/note",
        "chgrp ops /usr/local/bin/report",
        "umask 077",
    ] {
        let mut current = (*d.initial).clone();
        let before = current.clone();
        assert!(
            current.execute_in_context(&d.initial, command).is_err(),
            "{command}"
        );
        assert_eq!(
            current, before,
            "no side effects survive refusal: {command}"
        );
    }
}
#[test]
fn source_context_erasure_caller_and_file_forgery_refuse_before_commands() {
    let d = definition();
    for kind in 0..6 {
        let mut current = (*d.initial).clone();
        match kind {
            0 => current.command_location = None,
            1 => current.command_location.as_mut().unwrap().source_id = "foreign".into(),
            2 => current.user = "blair".into(),
            3 => {
                current.users.get_mut("alex").unwrap().pop();
            }
            4 => current.home = "/work".into(),
            _ => {
                current
                    .files
                    .insert("/work/note".into(), "forged body\n".into());
            }
        }
        let before = current.clone();
        assert!(policy::bound(&d.initial, &current).is_err());
        assert!(current.execute_in_context(&d.initial, "pwd").is_err());
        assert_eq!(current, before);
        assert!(!d.check(&current).0);
    }
}
#[test]
fn unique_reference_source_matrix_never_chooses_first_or_falls_back() {
    let d = definition();
    let rows = vec![requirement(d.clone())];
    assert!(policy::for_current(&rows, &d.initial).unwrap().is_some());
    let mut erased = (*d.initial).clone();
    erased.command_location = None;
    assert!(policy::for_current(&rows, &erased).is_err());
    let old = vec![Requirement::Mode {
        path: "/usr/local/bin/report".into(),
        mode: 0o644,
    }];
    assert!(policy::for_current(&old, &d.initial).is_err());
    assert!(policy::for_current(&old, &erased).unwrap().is_none());
    assert!(policy::unique(&[requirement(d.clone()), requirement(d)]).is_err());
}
#[test]
fn readonly_profile_refuses_operational_role_but_projection_is_nonrecursive() {
    let d = definition();
    let context = d.initial.command_location.as_ref().unwrap();
    assert!(d.initial.valid());
    context.validate_for(&d.initial).unwrap();
    let readonly = Profile {
        version: 1,
        tree: (*d.initial).clone(),
        trusted_directories: vec![],
        names: vec!["report".into()],
    };
    assert_eq!(
        readonly.validate().unwrap_err().kind,
        path_lookup::ErrorKind::Invalid
    );
    let output = dlms_learning_lab::command_location::evaluate(
        context,
        &d.initial,
        &d.expected,
        &Query {
            mode: Mode::Which,
            name: "report".into(),
        },
    )
    .unwrap();
    assert_eq!(output.paths, ["/usr/local/bin/report"]);
}
#[test]
fn definition_requires_matching_complete_source_and_accurate_selection_rows() {
    let good = definition();
    for kind in 0..6 {
        let mut d = good.clone();
        match kind {
            0 => d.selected[0].path = "/usr/bin/report".into(),
            1 => d.selected[1].name = "report".into(),
            2 => {
                d.selected.pop();
            }
            3 => d.expected.command_location.as_mut().unwrap().source_id = "different".into(),
            4 => {
                d.expected
                    .files
                    .insert("/work/note".into(), "not preserved\n".into());
            }
            _ => d.expected.nodes.get_mut("/work/note").unwrap().mode = 0o600,
        }
        assert!(d.validate().is_err());
    }
    let mut other = (*good.initial).clone();
    other.clock = 1;
    assert!(
        !good.valid_for(&other),
        "even original timestamp declaration is pinned"
    );
}
#[test]
fn malformed_expected_owner_kind_directory_mode_and_new_path_are_refused() {
    let good = definition();
    for kind in 0..5 {
        let mut d = good.clone();
        match kind {
            0 => {
                d.expected
                    .nodes
                    .get_mut("/usr/local/bin/report")
                    .unwrap()
                    .owner = "blair".into()
            }
            1 => {
                d.expected
                    .nodes
                    .get_mut("/usr/local/bin/report")
                    .unwrap()
                    .group = "ops".into()
            }
            2 => d.expected.nodes.get_mut("/work").unwrap().mode = 0o700,
            3 => {
                d.expected
                    .nodes
                    .insert("/work/new".into(), node(true, 0o700, "alex", "staff"));
            }
            _ => {
                d.expected.nodes.get_mut("/work/note").unwrap().directory = true;
                d.expected.files.remove("/work/note");
            }
        }
        assert!(d.validate().is_err());
    }
}
#[test]
fn definition_json_strictness_selection_count_and_size_are_bounded() {
    let d = definition();
    let raw = serde_json::to_vec(&d).unwrap();
    assert_eq!(Definition::decode(&raw).unwrap(), d);
    let mut value = serde_json::to_value(&d).unwrap();
    value["extra"] = true.into();
    assert!(Definition::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = serde_json::to_value(&d).unwrap();
    value["selected"] = serde_json::json!([
        d.selected[0].clone(),
        d.selected[1].clone(),
        d.selected[0].clone(),
        d.selected[1].clone(),
        d.selected[0].clone()
    ]);
    assert!(Definition::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = serde_json::to_value(&d).unwrap();
    value["selected"][0]["path"] = serde_json::json!("/".to_owned() + &"x".repeat(1024));
    assert!(Definition::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    assert!(Definition::decode(&vec![b' '; policy::MAX_DEFINITION_BYTES + 1]).is_err());
}
#[test]
fn valid_tree_content_can_still_exceed_scoped_definition_serialized_budget() {
    let mut d = definition();
    let text = "\u{1}".repeat(131072);
    d.initial
        .files
        .insert("/usr/local/bin/report".into(), text.clone());
    d.expected
        .files
        .insert("/usr/local/bin/report".into(), text);
    assert!(d.initial.valid());
    assert!(d.expected.valid());
    assert!(d.validate().unwrap_err().contains("1MiB"));
}
#[test]
fn known_missing_has_diagnostic_while_unknown_query_or_mode_is_unsupported() {
    let mut source = initial();
    source
        .command_location
        .as_mut()
        .unwrap()
        .names
        .push("missing".into());
    let mut current = source.clone();
    let missing = current
        .execute_in_context(&source, "which missing")
        .unwrap_err();
    assert!(missing.starts_with("Simulator lookup:"));
    let unknown = current
        .execute_in_context(&source, "which not-supplied")
        .unwrap_err();
    assert!(unknown.contains("not a supplied"));
    assert!(
        current
            .execute_in_context(&source, "type -P report")
            .unwrap_err()
            .contains("Supported queries")
    );
    assert!(
        current
            .execute_in_context(&source, "which report; rm /work/note")
            .is_err()
    );
    assert_eq!(current, source);
}
#[test]
fn late_valid_permission_change_invalidates_then_repair_restores_current_grade() {
    let d = definition();
    let mut current = (*d.expected).clone();
    assert!(d.check(&current).0);
    current
        .execute_in_context(&d.initial, "chmod u-x /usr/local/bin/report")
        .unwrap();
    assert!(!d.check(&current).0);
    current
        .execute_in_context(&d.initial, "chmod u+x /usr/local/bin/report")
        .unwrap();
    assert!(d.check(&current).0);
}
