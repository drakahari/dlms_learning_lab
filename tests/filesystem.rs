use dlms_learning_lab::{
    engine::{self, Goal, Node, State},
    filesystem::{Requirement, Tree},
    pack,
    progress::{self, Attempt, Progress},
};
use std::collections::BTreeMap;
fn node(directory: bool, mode: u16, owner: &str) -> Node {
    Node {
        directory,
        mode,
        owner: owner.into(),
        group: if owner == "root" { "root" } else { "staff" }.into(),
    }
}
fn tree() -> Tree {
    Tree {
        nodes: BTreeMap::from([
            ("/".into(), node(true, 0o755, "root")),
            ("/home".into(), node(true, 0o755, "root")),
            ("/home/alex".into(), node(true, 0o700, "alex")),
            ("/home/alex/source".into(), node(false, 0o640, "alex")),
            ("/home/alex/.hidden".into(), node(false, 0o600, "alex")),
            ("/home/alex/other".into(), node(true, 0o750, "alex")),
            ("/home/alex/locked".into(), node(true, 0o000, "alex")),
        ]),
        files: BTreeMap::from([
            ("/home/alex/source".into(), "original\n".into()),
            ("/home/alex/.hidden".into(), "private\n".into()),
        ]),
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into()]),
            ("blair".into(), vec!["staff".into()]),
        ]),
        user: "alex".into(),
        home: "/home/alex".into(),
        cwd: "/home/alex".into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
    }
}
#[test]
fn navigation_quoting_hidden_names_and_option_boundaries_are_real_actions() {
    let mut t = tree();
    assert!(t.valid());
    assert_eq!(t.execute("pwd").unwrap(), "/home/alex\n");
    t.execute("mkdir 'two words'").unwrap();
    t.execute("cd 'two words'").unwrap();
    assert_eq!(t.cwd, "/home/alex/two words");
    t.execute("touch -- -n").unwrap();
    assert_eq!(t.execute("ls").unwrap(), "-n\n");
    let before = t.clone();
    assert!(t.execute("rm -f -n").is_err());
    assert_eq!(t, before);
    t.execute("rm -f -- -n").unwrap();
    assert!(!t.nodes.contains_key("/home/alex/two words/-n"));
    assert_eq!(t.execute("cd -").unwrap(), "/home/alex\n");
    assert!(!t.execute("ls").unwrap().contains(".hidden"));
    assert!(t.execute("ls -a").unwrap().starts_with(".\n..\n.hidden\n"));
    t.execute("mkdir -- -P").unwrap();
    let before = t.clone();
    assert!(t.execute("cd -P").is_err());
    assert_eq!(t, before);
    t.execute("cd -- -P").unwrap();
    t.execute("cd").unwrap();
    assert_eq!(t.cwd, t.home);
}
#[test]
fn dotdot_cannot_skip_missing_non_directory_or_inaccessible_ancestors() {
    let mut t = tree();
    for input in [
        "cat locked/../source",
        "cat missing/../source",
        "cat source/../source",
        "cd locked/..",
    ] {
        let before = t.clone();
        assert!(t.execute(input).is_err(), "{input}");
        assert_eq!(t, before);
    }
    assert!(
        t.execute("cat locked/../source")
            .unwrap_err()
            .contains("Permission denied")
    );
    assert!(
        t.execute("cat missing/../source")
            .unwrap_err()
            .contains("No such")
    );
    assert!(
        t.execute("cat source/../source")
            .unwrap_err()
            .contains("Not a directory")
    );
    assert_eq!(t.execute("cat ./other/../source").unwrap(), "original\n");
    t.execute("cd /../../home/alex").unwrap();
    assert_eq!(t.cwd, "/home/alex");
}
#[test]
fn copy_and_move_have_distinct_permissions_ownership_and_umask_rules() {
    let mut t = tree();
    t.execute("umask 077").unwrap();
    t.execute("cp source copied").unwrap();
    assert_eq!(t.nodes["/home/alex/copied"].mode, 0o600);
    assert_eq!(t.files["/home/alex/source"], t.files["/home/alex/copied"]);
    t.nodes.get_mut("/home/alex/copied").unwrap().mode = 0o660;
    t.execute("cp source copied").unwrap();
    assert_eq!(t.nodes["/home/alex/copied"].mode, 0o660);
    t.nodes.get_mut("/home/alex/source").unwrap().mode = 0o000;
    assert!(t.execute("cp source denied").is_err());
    assert!(!t.nodes.contains_key("/home/alex/denied"));
    t.nodes.get_mut("/home/alex/source").unwrap().owner = "root".into();
    t.execute("mv source other").unwrap();
    assert_eq!(t.nodes["/home/alex/other/source"].mode, 0o000);
    assert_eq!(t.nodes["/home/alex/other/source"].owner, "root");
    assert!(!t.nodes.contains_key("/home/alex/source"));
    // Removing/renaming uses parent write+search; file write is not required.
    t.execute("rm other/source").unwrap();
    assert!(!t.nodes.contains_key("/home/alex/other/source"));
}
#[test]
fn parent_write_and_file_write_are_not_interchangeable() {
    let mut t = tree();
    t.nodes.get_mut("/home/alex").unwrap().mode = 0o500;
    assert!(t.execute("touch new").is_err());
    assert!(t.execute("mv source other").is_err());
    assert!(t.execute("rm source").is_err());
    t.files.insert("/home/alex/.hidden".into(), "old\n".into());
    t.execute("cp source .hidden").unwrap(); // existing writable file does not need parent write
    assert_eq!(t.files["/home/alex/.hidden"], "original\n");
    t.nodes.get_mut("/home/alex/.hidden").unwrap().mode = 0o400;
    assert!(t.execute("cp source .hidden").is_err());
    t.execute("touch .hidden").unwrap(); // ordinary owner may set current timestamps
}
#[test]
fn failed_commands_and_resource_limits_never_partially_apply() {
    let mut t = tree();
    for input in [
        "cp source source",
        "mv source source",
        "mkdir other",
        "cat /etc/passwd",
        "cat source/",
        "rm -r other",
        "printf boom > source",
        "touch x; rm source",
        "cat $(id)",
        "touch \"$HOME\"",
        "touch ../outside/thing",
    ] {
        let before = t.clone();
        assert!(t.execute(input).is_err(), "{input}");
        assert_eq!(t, before, "{input}");
    }
    t.clock = 1_000_000;
    let before = t.clone();
    assert!(t.execute("touch source").is_err());
    assert_eq!(t, before);
    t = tree();
    t.files
        .insert("/home/alex/source".into(), "x".repeat(131_072));
    assert!(t.valid());
    let before = t.clone();
    assert!(
        t.execute("cat source")
            .unwrap_err()
            .contains("Output limit")
    );
    assert_eq!(t, before);
    let before = t.clone();
    assert!(t.execute("cp source copy").is_err());
    assert_eq!(t, before);
    t.files
        .insert("/home/alex/source".into(), "x".repeat(131_068));
    t.execute("cp source copy").unwrap();
    assert_eq!(t.files.values().map(String::len).sum::<usize>(), 262_144);
    let before = t.clone();
    assert!(t.execute("cp source third").is_err());
    assert_eq!(t, before); // protected hidden data is included in memory
}
#[test]
fn directory_removal_and_state_validation_keep_tree_consistent() {
    let mut t = tree();
    t.execute("touch other/item").unwrap();
    assert!(t.execute("rmdir other").is_err());
    t.execute("rm other/item").unwrap();
    t.execute("rmdir other").unwrap();
    assert!(!t.nodes.contains_key("/home/alex/other"));
    for change in 0..5 {
        let mut invalid = tree();
        match change {
            0 => {
                invalid.nodes.remove("/");
            }
            1 => {
                invalid.files.insert("/outside".into(), "orphan".into());
            }
            2 => {
                invalid
                    .nodes
                    .get_mut("/home/alex/source")
                    .unwrap()
                    .directory = true;
            }
            3 => {
                invalid.user = "root".into();
            }
            _ => {
                invalid.modified.insert("/missing".into(), 1);
            }
        }
        assert!(!invalid.valid());
        assert!(pack::validate_state(&State::Filesystem { tree: invalid }).is_err());
    }
}
#[test]
fn typed_objectives_recompute_after_breakage_and_resume_without_touching_old_attempts() {
    let mut t = tree();
    let goal = Goal::Filesystem {
        requirements: vec![
            Requirement::Contents {
                path: "/home/alex/report".into(),
                text: "original\n".into(),
            },
            Requirement::Contents {
                path: "/home/alex/source".into(),
                text: "original\n".into(),
            },
        ],
    };
    assert!(!engine::solved(
        &State::Filesystem { tree: t.clone() },
        &goal
    ));
    t.execute("cp source report").unwrap();
    assert!(engine::solved(
        &State::Filesystem { tree: t.clone() },
        &goal
    ));
    t.execute("rm report").unwrap();
    assert!(!engine::solved(
        &State::Filesystem { tree: t.clone() },
        &goal
    ));
    let mut lab = pack::bundled().unwrap()[3].clone();
    lab.legacy_access_teaching = None; // This fixture replaces Unix with Filesystem.
    lab.id = "test-files".into();
    lab.initial = State::Filesystem { tree: t };
    lab.goal = goal;
    lab.solution = vec!["cp source report".into()];
    lab.prerequisites.clear();
    let mut labs = pack::bundled().unwrap();
    labs.push(lab.clone());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), Attempt::new(&lab, 1));
    p.attempts
        .insert(labs[0].id.clone(), Attempt::new(&labs[0], 2));
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let resumed = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(resumed).unwrap()
    );
    assert!(
        !Requirement::Contents {
            path: "/bad/../escape".into(),
            text: "x".into()
        }
        .valid()
    );
}

#[test]
fn already_open_cwd_does_not_recheck_ancestors_but_actual_traversal_does() {
    let mut t = tree();
    t.nodes
        .insert("/home/alex/locked/inner".into(), node(true, 0o755, "alex"));
    t.nodes.insert(
        "/home/alex/locked/inner/note".into(),
        node(false, 0o644, "alex"),
    );
    t.files
        .insert("/home/alex/locked/inner/note".into(), "visible\n".into());
    t.cwd = "/home/alex/locked/inner".into();
    assert!(t.valid());
    assert_eq!(t.execute("cat note").unwrap(), "visible\n");
    assert_eq!(t.execute("pwd").unwrap(), "/home/alex/locked/inner\n");
    assert!(t.execute("cat /home/alex/locked/inner/note").is_err());
    assert!(t.execute("ls ..").is_err());
    t.execute("touch new").unwrap();
    t.execute("cp note copy").unwrap();
    t.execute("rm new").unwrap();
}
#[test]
fn force_and_dash_operands_keep_real_special_cases() {
    let mut t = tree();
    for command in [
        "rm -f absent/leaf",
        "rm -f source/leaf",
        "rm -f absent/",
        "rm -f source/",
    ] {
        t.execute(command).unwrap();
    }
    assert!(
        t.execute("rm -f locked/leaf")
            .unwrap_err()
            .contains("Permission denied")
    );
    for command in ["cat -- -", "touch -- -"] {
        assert!(
            t.execute(command).unwrap_err().contains("unsupported")
                || t.execute(command).unwrap_err().contains("does not support")
        );
    }
    t.execute("touch ./-").unwrap();
    t.execute("cat ./-").unwrap();
    t.execute("cp -- - copied").unwrap();
    t.execute("touch -- -f").unwrap();
    t.execute("rm -- -f").unwrap();
    assert!(!t.nodes.contains_key("/home/alex/-f"));
    t.execute("mkdir trailing/").unwrap();
    t.execute("cd trailing").unwrap();
    assert_eq!(t.execute("cd -- -").unwrap(), "/home/alex\n");
}

#[test]
fn removed_previous_directory_remains_a_path_but_cd_reports_missing() {
    let mut t = tree();
    t.execute("cd other").unwrap();
    t.execute("cd ..").unwrap();
    t.execute("rmdir other").unwrap();
    assert!(t.valid());
    assert!(t.execute("cd -").unwrap_err().contains("No such"));
}
#[test]
fn entry_and_path_limits_and_invalid_goals_are_checked() {
    let mut t = tree();
    while t.nodes.len() < 256 {
        let name = format!("f{}", t.nodes.len());
        t.execute(&format!("touch {name}")).unwrap();
    }
    let before = t.clone();
    assert!(t.execute("touch excess").is_err());
    assert_eq!(t, before);
    assert!(t.execute(&format!("touch {}", "x".repeat(256))).is_err());
    assert!(t.execute(&format!("cat {}", "x/".repeat(65))).is_err());
    let mut lab = pack::bundled().unwrap()[3].clone();
    lab.legacy_access_teaching = None; // This fixture replaces Unix with Filesystem.
    lab.initial = State::Filesystem { tree: tree() };
    lab.goal = Goal::Filesystem {
        requirements: vec![Requirement::Entry {
            path: "/bad/../path".into(),
            directory: true,
        }],
    };
    lab.solution = vec!["pwd".into()];
    assert!(pack::validate_state(&lab.initial).is_ok());
    // Full pack validation, not only state validation, rejects malformed objectives.
    let mut value = serde_json::to_value(pack::Pack {
        schema_version: 1,
        scenarios: vec![],
    })
    .unwrap();
    value["scenarios"] = serde_json::json!([lab]);
    assert!(pack::load(&value.to_string()).is_err());
}

#[test]
fn scripting_globs_do_not_change_the_separate_literal_terminal_language() {
    let mut t = tree();
    t.execute("touch '*.log'").unwrap();
    t.execute(r"touch x\?.txt").unwrap();
    for command in [
        "touch *.new",
        "rm *",
        "rm *.log",
        "ls ?",
        "touch 'quoted'*",
        "rm x?.txt",
    ] {
        let before = t.clone();
        let error = t.execute(command).unwrap_err();
        assert!(
            error.contains("Filename expansion is unsupported"),
            "{command}: {error}"
        );
        assert_eq!(t, before);
    }
    t.execute(r"rm \*.log").unwrap();
    t.execute("rm 'x?.txt'").unwrap();
    assert!(!t.nodes.contains_key("/home/alex/*.log"));
    assert!(!t.nodes.contains_key("/home/alex/x?.txt"));
}
