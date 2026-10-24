//! Exact current folder inventory; no find-command or transcript requirement.
use dlms_learning_lab::{
    engine::{self, Goal, Node, State},
    filesystem::{Access, Requirement, Tree},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    terminal_teaching, workspace,
};
use std::collections::BTreeMap;

fn node(directory: bool) -> Node {
    Node {
        directory,
        mode: if directory { 0o700 } else { 0o600 },
        owner: "alex".into(),
        group: "staff".into(),
    }
}
fn add(tree: &mut Tree, path: &str, directory: bool, text: &str) {
    tree.nodes.insert(path.into(), node(directory));
    if !directory {
        tree.files.insert(path.into(), text.into());
    }
}
fn tree() -> Tree {
    let mut t = Tree {
        nodes: BTreeMap::new(),
        files: BTreeMap::new(),
        users: BTreeMap::from([("alex".into(), vec!["staff".into()])]),
        user: "alex".into(),
        home: "/work".into(),
        cwd: "/work".into(),
        previous: None,
        umask: 0o077,
        clock: 0,
        modified: BTreeMap::new(),
    };
    for path in [
        "/",
        "/work",
        "/work/review",
        "/work/review/archive",
        "/work/empty",
    ] {
        add(&mut t, path, true, "");
    }
    for (path, text) in [
        ("/work/review/report.txt", "approved report\n"),
        ("/work/review/.receipt", "approved receipt\n"),
        ("/work/review/archive/old.txt", "older nested record\n"),
        ("/work/private.txt", "private original\n"),
    ] {
        add(&mut t, path, false, text);
    }
    assert!(t.valid());
    t
}
fn entries(path: &str, names: &[&str]) -> Requirement {
    Requirement::DirectoryEntries {
        path: path.into(),
        names: names.iter().map(|s| (*s).into()).collect(),
    }
}
fn expected() -> Requirement {
    entries("/work/review", &["report.txt", "archive", ".receipt"])
}

#[test]
fn immediate_names_are_unordered_literal_case_sensitive_and_include_hidden_entries() {
    let mut t = tree();
    let before = t.clone();
    assert!(expected().valid_for(&t) && expected().check(&t).0);
    assert!(
        entries("/work/review", &[".receipt", "archive", "report.txt"])
            .check(&t)
            .0
    );
    assert!(
        !entries("/work/review", &["report.txt", "archive"])
            .check(&t)
            .0
    );
    assert!(
        !entries("/work/review", &["Report.txt", "archive", ".receipt"])
            .check(&t)
            .0
    );
    assert_eq!(t, before);
    t.execute("touch /work/review/archive/unmentioned-nested.txt")
        .unwrap();
    assert!(expected().check(&t).0, "Only immediate names are assessed");
    t.execute("touch '/work/review/*'").unwrap();
    t.execute("touch '/work/review/[x]'").unwrap();
    assert!(!expected().check(&t).0);
    assert!(
        entries(
            "/work/review",
            &["report.txt", "archive", ".receipt", "*", "[x]"]
        )
        .check(&t)
        .0
    );
    assert!(
        !entries("/work/review", &["*"]).check(&t).0,
        "Expected names are literal, never wildcard patterns"
    );
}

#[test]
fn empty_expectation_requires_a_real_valid_directory_and_does_not_invent_access() {
    let mut t = tree();
    assert!(entries("/work/empty", &[]).check(&t).0);
    assert!(!entries("/work/missing", &[]).check(&t).0);
    assert!(!entries("/work/private.txt", &[]).check(&t).0);
    t.nodes.get_mut("/work/empty").unwrap().mode = 0;
    assert!(
        entries("/work/empty", &[]).check(&t).0,
        "Inventory is structural; required listing access is separate"
    );
    assert!(
        !Requirement::Access {
            path: "/work/empty".into(),
            user: "alex".into(),
            operation: Access::ListNames,
            allowed: true
        }
        .check(&t)
        .0
    );
    let mut invalid = t.clone();
    invalid.files.insert("/unmodeled".into(), "invalid".into());
    assert!(!entries("/work/empty", &[]).check(&invalid).0);
}

#[test]
fn extra_private_names_missing_names_and_repair_are_rechecked_from_current_state() {
    let mut t = tree();
    assert!(expected().check(&t).0);
    t.execute("cp /work/private.txt /work/review/renamed-unexpected.txt")
        .unwrap();
    let (pass, feedback) = expected().check(&t);
    assert!(!pass && feedback.contains("0 missing, 1 unrequested"));
    assert!(!feedback.contains("renamed-unexpected.txt") && feedback.contains("ls -a"));
    t.execute("rm /work/review/renamed-unexpected.txt").unwrap();
    assert!(expected().check(&t).0);
    t.execute("mv /work/review/.receipt /work/saved-receipt")
        .unwrap();
    let (pass, feedback) = expected().check(&t);
    assert!(!pass && feedback.contains("1 missing, 0 unrequested"));
    assert!(!feedback.contains(".receipt"));
    t.execute("mv /work/saved-receipt /work/review/.receipt")
        .unwrap();
    assert!(expected().check(&t).0);
    assert_eq!(t.files["/work/private.txt"], "private original\n");
}

#[test]
fn names_and_paths_validate_bounds_duplicates_components_and_utf8_without_state_changes() {
    let t = tree();
    let before = serde_json::to_vec(&t).unwrap();
    for name in ["", ".", "..", "a/b", "a\nb", "a\0b"] {
        let requirement = entries("/work/review", &[name]);
        assert!(!requirement.valid() && !requirement.check(&t).0, "{name:?}");
    }
    assert!(!entries("/work/review", &["same", "same"]).valid());
    assert!(!entries("/work/../review", &[]).valid());
    assert!(!entries("/work/review", &[&"x".repeat(256)]).valid());
    assert!(
        entries(
            "/work/review",
            &[
                &format!("{}a", "é".repeat(127)),
                "two words",
                "?",
                ".hidden"
            ]
        )
        .valid()
    );
    let maximum = Requirement::DirectoryEntries {
        path: "/work/review".into(),
        names: (0..256).map(|n| format!("n{n}")).collect(),
    };
    assert!(maximum.valid());
    let too_many = Requirement::DirectoryEntries {
        path: "/work/review".into(),
        names: (0..257).map(|n| format!("n{n}")).collect(),
    };
    assert!(!too_many.valid() && !too_many.check(&t).0);
    let huge_path = Requirement::DirectoryEntries {
        path: "x".repeat(4096),
        names: vec![],
    };
    let (pass, message) = huge_path.check(&t);
    assert!(!pass && message.len() < 100);
    assert_eq!(serde_json::to_vec(&t).unwrap(), before);
    let mut long_tree = t.clone();
    let mut long_path = "/work".to_string();
    for _ in 0..6 {
        long_path.push('/');
        long_path.push_str(&"é".repeat(60));
        add(&mut long_tree, &long_path, true, "");
    }
    let many_names = Requirement::DirectoryEntries {
        path: long_path,
        names: (0..256)
            .map(|n| format!("{}{:03}", "x".repeat(252), n))
            .collect(),
    };
    assert!(many_names.valid());
    let (pass, feedback) = many_names.check(&long_tree);
    assert!(!pass && feedback.contains("256 missing, 0 unrequested") && feedback.contains("ls -a"));
    assert!(feedback.len() <= 512 && feedback.capacity() <= 512);
    assert!(!feedback.contains(&"x".repeat(252)));
    let json = serde_json::to_string(&expected()).unwrap();
    let decoded: Requirement = serde_json::from_str(&json).unwrap();
    assert!(decoded.check(&t).0);
    assert!(
        serde_json::from_str::<Requirement>(
            r#"{"kind":"DirectoryEntries","path":"/work","names":[],"extra":true}"#
        )
        .is_err()
    );
}

#[test]
fn field_decoder_bounds_typed_names_and_preserves_tag_order_and_legacy_wire() {
    let valid_names: Vec<_> = (0..256)
        .map(|n| serde_json::json!(format!("n{n}")))
        .collect();
    let accepted =
        serde_json::json!({"kind":"DirectoryEntries", "path":"/work/review", "names":valid_names});
    assert!(
        serde_json::from_value::<Requirement>(accepted)
            .unwrap()
            .valid()
    );
    let mut excess: Vec<_> = (0..256)
        .map(|n| serde_json::json!(format!("n{n}")))
        .collect();
    // Valid JSON, deliberately not a string. Serde's existing outer tagged
    // Content is already buffered; our field Seed must report the count before
    // attempting typed String deserialization of this257th value.
    excess.push(serde_json::json!({"body":["a valid JSON body", {"nested":true}]}));
    let names_json = serde_json::to_string(&excess).unwrap();
    for raw in [
        format!(r#"{{"kind":"DirectoryEntries","path":"/work/review","names":{names_json}}}"#),
        format!(r#"{{"names":{names_json},"path":"/work/review","kind":"DirectoryEntries"}}"#),
    ] {
        assert!(serde_json::from_str::<serde_json::Value>(&raw).is_ok());
        let error = serde_json::from_str::<Requirement>(&raw)
            .unwrap_err()
            .to_string();
        assert!(error.contains("name count exceeds256"), "{error}");
        assert!(!error.contains("invalid type"));
    }
    for (names, fragment) in [
        (
            serde_json::json!(["duplicate", "duplicate"]),
            "Duplicate directory entry name",
        ),
        (
            serde_json::json!(["x".repeat(256)]),
            "Invalid directory entry name",
        ),
    ] {
        let raw =
            serde_json::json!({"kind":"DirectoryEntries", "path":"/work/review", "names":names});
        assert!(
            serde_json::from_value::<Requirement>(raw.clone())
                .unwrap_err()
                .to_string()
                .contains(fragment)
        );
        // Whole-pack parsing still rejects it; no pack-wide streaming claim.
        let mut content = serde_json::to_value(pack::Pack {
            schema_version: 1,
            scenarios: vec![lab()],
        })
        .unwrap();
        content["scenarios"][0]["goal"]["requirements"][0] = raw;
        assert!(
            pack::load(&content.to_string())
                .unwrap_err()
                .contains(fragment)
        );
    }
    let names = serde_json::json!(["z", ".hidden", "two words"]);
    for raw in [
        format!(r#"{{"kind":"DirectoryEntries","path":"/work/review","names":{names}}}"#),
        format!(r#"{{"names":{names},"path":"/work/review","kind":"DirectoryEntries"}}"#),
    ] {
        let requirement: Requirement = serde_json::from_str(&raw).unwrap();
        assert!(requirement.valid());
        assert_eq!(serde_json::to_value(requirement).unwrap()["names"], names);
    }
    for raw in [
        r#"{"kind":"Mode","path":"/work/private.txt","mode":384}"#,
        r#"{"mode":384,"path":"/work/private.txt","kind":"Mode"}"#,
        r#"{"allowed":true,"operation":"read_file","user":"alex","path":"/work/private.txt","kind":"Access"}"#,
    ] {
        let requirement: Requirement = serde_json::from_str(raw).unwrap();
        assert!(requirement.check(&tree()).0);
        assert_eq!(
            serde_json::to_value(requirement).unwrap(),
            serde_json::from_str::<serde_json::Value>(raw).unwrap()
        );
    }
}

fn lab() -> pack::Scenario {
    let mut lab = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == "lpic1-file-navigation")
        .unwrap();
    lab.id = "test-exact-directory-entries".into();
    lab.prerequisites.clear();
    let mut t = tree();
    add(
        &mut t,
        "/work/review/private-copy.txt",
        false,
        "private original\n",
    );
    lab.initial = State::Filesystem { tree: t };
    lab.goal = Goal::Filesystem {
        requirements: vec![
            expected(),
            Requirement::Contents {
                path: "/work/review/report.txt".into(),
                text: "approved report\n".into(),
            },
            Requirement::Contents {
                path: "/work/review/.receipt".into(),
                text: "approved receipt\n".into(),
            },
            Requirement::Contents {
                path: "/work/private.txt".into(),
                text: "private original\n".into(),
            },
            Requirement::Mode {
                path: "/work/private.txt".into(),
                mode: 0o600,
            },
        ],
    };
    lab.solution = vec!["rm /work/review/private-copy.txt".into()];
    lab
}

#[test]
fn full_goal_accepts_manual_alternates_but_rejects_renamed_leaks_and_overwritten_public_data() {
    let lab = lab();
    for commands in [
        vec!["rm /work/review/private-copy.txt"],
        vec!["mv /work/review/private-copy.txt /work/recovered-private.txt"],
    ] {
        let mut state = lab.initial.clone();
        for command in commands {
            engine::execute_for_goal(&mut state, &lab.goal, command).unwrap();
        }
        assert!(engine::solved(&state, &lab.goal));
    }
    for commands in [
        vec!["mv /work/review/private-copy.txt /work/review/another-name"],
        vec![
            "cp /work/review/private-copy.txt /work/review/report.txt",
            "rm /work/review/private-copy.txt",
        ],
        vec!["rm /work/review/report.txt"],
    ] {
        let mut state = lab.initial.clone();
        for command in commands {
            engine::execute_for_goal(&mut state, &lab.goal, command).unwrap();
        }
        assert!(!engine::solved(&state, &lab.goal));
        assert!(!workspace::issues(&state, &lab.goal).is_empty());
    }
    let encoded = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![lab.clone()],
    })
    .unwrap();
    assert_eq!(pack::load(&encoded).unwrap().len(), 1);
    let mut bad = lab;
    bad.goal = Goal::Filesystem {
        requirements: vec![entries("/work/review", &["dup", "dup"])],
    };
    assert!(
        pack::load(
            &serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: vec![bad]
            })
            .unwrap()
        )
        .is_err()
    );
}

#[test]
fn exact_inventory_handles_the_largest_valid_tree_and_does_not_count_root_as_its_own_child() {
    let mut t = tree();
    t.nodes.retain(|path, _| path == "/");
    t.files.clear();
    t.cwd = "/".into();
    t.home = "/".into();
    let names: Vec<_> = (0..255).map(|n| format!("f{n:03}")).collect();
    for name in &names {
        add(&mut t, &format!("/{name}"), false, "");
    }
    assert!(t.valid() && t.nodes.len() == 256);
    let requirement = Requirement::DirectoryEntries {
        path: "/".into(),
        names,
    };
    assert!(requirement.check(&t).0);
    assert!(!entries("/", &[]).check(&t).0);
}

#[test]
fn save_load_current_invalidation_and_selected_reset_keep_independent_drafts_and_other_attempts() {
    let mut labs = pack::bundled().unwrap();
    let other = labs
        .iter()
        .find(|s| s.id == "lpic1-file-navigation")
        .unwrap()
        .clone();
    let lab = lab();
    labs.push(lab.clone());
    let mut attempt = Attempt::new(&lab, 1);
    workspace::prepare(&mut attempt);
    let lesson = lab.terminal_teaching.as_ref().unwrap();
    terminal_teaching::prepare(&mut attempt, lesson, true);
    engine::execute_for_goal(&mut attempt.state, &lab.goal, &lab.solution[0]).unwrap();
    attempt.workspace.as_mut().unwrap().command = "cp 'unfinished assignment".into();
    let learning = attempt
        .workspace
        .as_mut()
        .unwrap()
        .terminal_learning
        .as_mut()
        .unwrap();
    learning.command = "touch independent-practice".into();
    learning.execute();
    learning.command = "find . -name 'unfinished practice".into();
    learning.view = View::Example;
    assert!(progress::current_solved(&attempt, &lab));
    let selected_before = serde_json::to_value(&attempt).unwrap();
    let mut unrelated = Attempt::new(&other, 2);
    workspace::prepare(&mut unrelated);
    unrelated.workspace.as_mut().unwrap().command = "retained other assignment".into();
    let other_before = serde_json::to_value(&unrelated).unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(lab.id.clone());
    p.favorites.insert(other.id.clone());
    p.attempts.insert(lab.id.clone(), attempt);
    p.attempts.insert(other.id.clone(), unrelated);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let raw = std::fs::read(&path).unwrap();
    let mut loaded = progress::load(&path, &labs).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), raw);
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&lab.id]).unwrap(),
        selected_before
    );
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&other.id]).unwrap(),
        other_before
    );
    assert!(progress::current_solved(&loaded.attempts[&lab.id], &lab));
    let practice_before = loaded.attempts[&lab.id]
        .workspace
        .as_ref()
        .unwrap()
        .terminal_learning
        .clone();
    engine::execute_for_goal(
        &mut loaded.attempts.get_mut(&lab.id).unwrap().state,
        &lab.goal,
        "touch /work/review/.unexpected",
    )
    .unwrap();
    assert!(!progress::current_solved(&loaded.attempts[&lab.id], &lab));
    assert_eq!(
        serde_json::to_value(
            &loaded.attempts[&lab.id]
                .workspace
                .as_ref()
                .unwrap()
                .terminal_learning
        )
        .unwrap(),
        serde_json::to_value(&practice_before).unwrap()
    );
    engine::execute_for_goal(
        &mut loaded.attempts.get_mut(&lab.id).unwrap().state,
        &lab.goal,
        "rm /work/review/.unexpected",
    )
    .unwrap();
    assert!(progress::current_solved(&loaded.attempts[&lab.id], &lab));
    let mut reset = Attempt::new(&lab, 3);
    workspace::prepare(&mut reset);
    terminal_teaching::prepare(&mut reset, lesson, true);
    loaded.attempts.insert(lab.id.clone(), reset);
    assert!(!progress::current_solved(&loaded.attempts[&lab.id], &lab));
    assert!(
        loaded.attempts[&lab.id]
            .workspace
            .as_ref()
            .unwrap()
            .command
            .is_empty()
    );
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&other.id]).unwrap(),
        other_before
    );
    assert!(loaded.favorites.contains(&lab.id) && loaded.favorites.contains(&other.id));
    progress::save(&path, &loaded).unwrap();
    let resumed = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&resumed.attempts).unwrap(),
        serde_json::to_value(&loaded.attempts).unwrap()
    );
}
