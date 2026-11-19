//! Opt-in recursive foundation: virtual outcomes, bounds and legacy isolation.
use dlms_learning_lab::{
    engine::Node,
    filesystem::{
        Tree,
        recursive::{self as core, ErrorKind, Operation},
    },
};
use std::collections::BTreeMap;
fn node(dir: bool, mode: u16, owner: &str, group: &str) -> Node {
    Node {
        directory: dir,
        mode,
        owner: owner.into(),
        group: group.into(),
    }
}
fn tree() -> Tree {
    Tree {
        nodes: BTreeMap::from([
            ("/".into(), node(true, 0o755, "root", "root")),
            ("/home".into(), node(true, 0o755, "root", "root")),
            ("/home/alex".into(), node(true, 0o700, "alex", "staff")),
            ("/work".into(), node(true, 0o755, "alex", "staff")),
            ("/work/source".into(), node(true, 0o755, "bea", "team")),
            (
                "/work/source/readme".into(),
                node(false, 0o644, "bea", "team"),
            ),
            (
                "/work/source/.hidden".into(),
                node(false, 0o644, "bea", "team"),
            ),
            (
                "/work/source/empty".into(),
                node(true, 0o750, "alex", "staff"),
            ),
            ("/work/source/sub".into(), node(true, 0o755, "bea", "team")),
            (
                "/work/source/sub/data".into(),
                node(false, 0o644, "bea", "team"),
            ),
            ("/work/keep".into(), node(false, 0o600, "alex", "staff")),
            ("/work/stage".into(), node(true, 0o755, "alex", "staff")),
        ]),
        files: BTreeMap::from([
            ("/work/source/readme".into(), "original\n".into()),
            ("/work/source/.hidden".into(), "hidden without LF".into()),
            ("/work/source/sub/data".into(), "café\n\n".into()),
            ("/work/keep".into(), "retain exact private text\n".into()),
        ]),
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into()]),
            ("bea".into(), vec!["team".into(), "staff".into()]),
        ]),
        user: "alex".into(),
        home: "/home/alex".into(),
        cwd: "/work".into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: None,
    }
}
fn copy(t: &mut Tree, source: &str, destination: &str) -> Result<(), core::Error> {
    core::apply(
        t,
        &Operation::Copy {
            source: source.into(),
            destination: destination.into(),
        },
    )
}
fn remove(t: &mut Tree, path: &str) -> Result<(), core::Error> {
    core::apply(t, &Operation::Remove { path: path.into() })
}
fn unchanged_error(t: &mut Tree, operation: Operation, kind: ErrorKind) {
    let old = t.clone();
    let e = core::apply(t, &operation).unwrap_err();
    assert_eq!(e.kind, kind, "{e}");
    assert_eq!(*t, old);
}
#[test]
fn recursive_copy_includes_nested_empty_hidden_names_and_exact_utf8_text() {
    let mut t = tree();
    assert!(t.valid());
    let old = t.clone();
    copy(&mut t, "source", "copy").unwrap();
    assert!(t.valid());
    for (p, n) in &old.nodes {
        assert_eq!(t.nodes[p], *n);
        if p.starts_with("/work/source") {
            let new = p.replacen("/work/source", "/work/copy", 1);
            assert_eq!(t.nodes[&new].directory, n.directory);
            assert_eq!(t.nodes[&new].owner, "alex");
            assert_eq!(t.nodes[&new].group, "staff");
            assert_eq!(t.nodes[&new].mode, n.mode & !old.umask);
            if !n.directory {
                assert_eq!(t.files[&new], old.files[p]);
            }
        }
    }
    assert_eq!(t.files["/work/keep"], old.files["/work/keep"]);
    assert!(t.nodes["/work/copy/empty"].directory);
    assert_eq!(t.files["/work/copy/.hidden"], "hidden without LF");
    assert_eq!(t.files["/work/copy/sub/data"], "café\n\n");
}
#[test]
fn existing_destination_directory_adds_source_basename_but_never_merges() {
    let mut t = tree();
    copy(&mut t, "source", "stage").unwrap();
    assert!(t.nodes.contains_key("/work/stage/source/sub/data"));
    let old = t.clone();
    assert_eq!(
        copy(&mut t, "source", "stage").unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(t, old);
}
#[test]
fn each_recursive_alias_and_literal_option_terminator_has_real_parsed_operands() {
    for flag in ["-R", "-r", "--recursive"] {
        let mut t = tree();
        core::execute(&mut t, &format!("cp {flag} -- source stage")).unwrap();
        assert_eq!(t.files["/work/stage/source/readme"], "original\n");
        core::execute(&mut t, &format!("rm {flag} -- stage/source")).unwrap();
        assert!(!t.nodes.contains_key("/work/stage/source"));
        assert!(t.nodes.contains_key("/work/source"));
    }
    let mut t = tree();
    core::execute(&mut t, "mkdir -- '-leading dir'").unwrap();
    core::execute(&mut t, "touch -- '-leading dir/file'").unwrap();
    core::execute(&mut t, "cp -R -- '-leading dir' 'two words'").unwrap();
    assert!(t.files.contains_key("/work/two words/file"));
}
#[test]
fn modes_use_source_bits_umask_and_temporary_owner_access_while_populating() {
    for mask in [0, 0o022, 0o077, 0o777] {
        let mut t = tree();
        t.umask = mask;
        t.nodes.get_mut("/work/source").unwrap().mode = 0o555;
        copy(&mut t, "source", "copy").unwrap();
        assert_eq!(t.nodes["/work/copy"].mode, 0o555 & !mask);
        assert_eq!(t.files["/work/copy/readme"], "original\n");
        assert_eq!(t.nodes["/work/copy/readme"].mode, 0o644 & !mask);
        assert_eq!(t.nodes["/work/source"].mode, 0o555);
    }
}
#[test]
fn ordinary_file_steps_are_an_equivalent_supported_copy_method() {
    let mut recursive = tree();
    copy(&mut recursive, "source", "copy").unwrap();
    let mut manual = tree();
    for c in [
        "mkdir copy",
        "mkdir copy/sub",
        "mkdir copy/empty",
        "chmod 750 copy/empty",
        "cp source/readme copy/readme",
        "cp source/.hidden copy/.hidden",
        "cp source/sub/data copy/sub/data",
    ] {
        core::execute(&mut manual, c).unwrap();
    }
    assert_eq!(recursive.nodes, manual.nodes);
    assert_eq!(recursive.files, manual.files);
    assert_eq!(recursive.users, manual.users);
    assert_eq!(recursive.umask, manual.umask);
    assert_ne!(
        recursive.modified, manual.modified,
        "Action counters are not copied-content correctness"
    );
}
#[test]
fn recursive_removal_keeps_unrelated_source_and_ignores_file_write_permission() {
    let mut t = tree();
    copy(&mut t, "source", "copy").unwrap();
    t.nodes.get_mut("/work/copy/readme").unwrap().mode = 0o400;
    let old = t.clone();
    remove(&mut t, "copy").unwrap();
    assert!(t.valid());
    assert!(t.nodes.keys().all(|p| !p.starts_with("/work/copy")));
    assert!(t.files.keys().all(|p| !p.starts_with("/work/copy")));
    assert!(t.modified.keys().all(|p| !p.starts_with("/work/copy")));
    for (p, n) in &old.nodes {
        if !p.starts_with("/work/copy") {
            assert_eq!(t.nodes[p], *n);
        }
    }
    for (p, data) in &old.files {
        if !p.starts_with("/work/copy") {
            assert_eq!(&t.files[p], data);
        }
    }
}
#[test]
fn parent_write_and_directory_search_read_failures_are_atomic_and_specific() {
    for (path, mode) in [
        ("/work/source/sub", 0o000),
        ("/work/source/readme", 0o000),
        ("/work/stage", 0o555),
    ] {
        let mut t = tree();
        t.nodes.get_mut(path).unwrap().mode = mode;
        unchanged_error(
            &mut t,
            Operation::Copy {
                source: "source".into(),
                destination: "stage".into(),
            },
            ErrorKind::Permission,
        );
    }
    let mut t = tree();
    copy(&mut t, "source", "copy").unwrap();
    t.nodes.get_mut("/work/copy/sub").unwrap().mode = 0o500;
    unchanged_error(
        &mut t,
        Operation::Remove {
            path: "copy".into(),
        },
        ErrorKind::Permission,
    );
}
#[test]
fn self_descendant_existing_file_and_final_dot_forms_are_honest_refusals() {
    for destination in ["source", "source/new", "/work"] {
        let mut t = tree();
        unchanged_error(
            &mut t,
            Operation::Copy {
                source: "source".into(),
                destination: destination.into(),
            },
            ErrorKind::Invalid,
        );
    }
    let mut t = tree();
    unchanged_error(
        &mut t,
        Operation::Copy {
            source: "source".into(),
            destination: "keep".into(),
        },
        ErrorKind::Unsupported,
    );
    for c in [
        "cp -R source/. copy",
        "cp -R source copy/",
        "cp -R source",
        "rm -r source other",
        "cp -a source copy",
        "rm -rf source",
        "cp -R source copy | cat",
        "cp -R source $(pwd)",
    ] {
        let mut t = tree();
        let old = t.clone();
        assert!(core::execute(&mut t, c).is_err(), "{c}");
        assert_eq!(t, old);
    }
}
#[test]
fn root_current_home_and_missing_paths_never_mutate_or_touch_host_files() {
    for p in ["/", "/work", "/home", "/home/alex"] {
        let mut t = tree();
        unchanged_error(
            &mut t,
            Operation::Remove { path: p.into() },
            ErrorKind::Unsupported,
        );
    }
    let mut t = tree();
    let old = t.clone();
    assert!(core::execute(&mut t, "cp -R ../../etc /work/out").is_err());
    assert_eq!(t, old);
    assert!(core::execute(&mut t, "rm -r /etc").is_err());
    assert_eq!(t, old);
    assert!(core::execute(&mut t, "curl https://example.test").is_err());
    assert_eq!(t, old);
}
#[test]
fn quotas_and_raw_inputs_reject_before_committing_state() {
    let mut t = tree();
    let current = t.clock;
    t.clock = 999_999;
    unchanged_error(
        &mut t,
        Operation::Copy {
            source: "source".into(),
            destination: "copy".into(),
        },
        ErrorKind::Limit,
    );
    t.clock = current;
    t.files
        .insert("/work/source/readme".into(), "x".repeat(131_072));
    t.files
        .insert("/work/source/sub/data".into(), "y".repeat(110_000));
    assert!(t.valid());
    unchanged_error(
        &mut t,
        Operation::Copy {
            source: "source".into(),
            destination: "copy".into(),
        },
        ErrorKind::Limit,
    );
    let mut t = tree();
    for i in 0..244 {
        let p = format!("/work/extra-{i}");
        t.nodes
            .insert(p.clone(), node(false, 0o600, "alex", "staff"));
        t.files.insert(p, String::new());
    }
    assert_eq!(t.nodes.len(), 256);
    assert!(t.valid());
    unchanged_error(
        &mut t,
        Operation::Copy {
            source: "source".into(),
            destination: "copy".into(),
        },
        ErrorKind::Limit,
    );
    let mut t = tree();
    let old = t.clone();
    assert_eq!(
        core::execute(&mut t, &"x".repeat(4097)).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(t, old);
}
#[test]
fn unchanged_legacy_tree_console_does_not_silently_gain_recursion() {
    let mut t = tree();
    let old = t.clone();
    assert!(t.execute("cp -R source copy").is_err());
    assert_eq!(t, old);
    assert!(t.execute("rm -r source").is_err());
    assert_eq!(t, old);
}
#[test]
fn editing_or_encoding_a_virtual_tree_never_creates_opaque_execution_authority() {
    let mut t = tree();
    core::execute(&mut t, "cp -R source copy").unwrap();
    let encoded = serde_json::to_vec(&t).unwrap();
    let mut restored: Tree = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(t, restored);
    core::execute(&mut restored, "rm -r copy").unwrap();
    assert!(!restored.nodes.contains_key("/work/copy"));
    assert_eq!(restored.files["/work/source/readme"], "original\n");
}
