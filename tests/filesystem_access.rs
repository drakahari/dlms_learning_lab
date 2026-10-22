use dlms_learning_lab::{
    engine::{self, Goal, Node, State},
    filesystem::{Access, Requirement, Tree},
    pack,
    progress::{self, Attempt, Progress},
    terminal_teaching,
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
fn tree() -> Tree {
    Tree {
        nodes: BTreeMap::from([
            ("/".into(), node(true, 0o755, "root", "root")),
            ("/work".into(), node(true, 0o755, "alex", "staff")),
            ("/work/source".into(), node(false, 0o600, "alex", "staff")),
            ("/work/foreign".into(), node(false, 0o600, "blair", "staff")),
            ("/work/locked".into(), node(true, 0o000, "alex", "staff")),
            (
                "/work/locked/note".into(),
                node(false, 0o666, "alex", "staff"),
            ),
            ("/work/dir".into(), node(true, 0o750, "alex", "staff")),
            ("/work/dir/note".into(), node(false, 0o640, "alex", "staff")),
        ]),
        files: BTreeMap::from([
            ("/work/source".into(), "original\n".into()),
            ("/work/foreign".into(), "another owner\n".into()),
            ("/work/locked/note".into(), "inaccessible\n".into()),
            ("/work/dir/note".into(), "team\n".into()),
        ]),
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into(), "review".into()]),
            ("blair".into(), vec!["staff".into()]),
            ("casey".into(), vec!["visitors".into()]),
        ]),
        user: "alex".into(),
        home: "/work".into(),
        cwd: "/work".into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
    }
}
#[test]
fn explicit_symbolic_modes_follow_numeric_bits_for_every_ordinary_starting_mode() {
    let specs = [
        ("u+r", 0o400, 0),
        ("g-w", 0, 0o020),
        ("o=x", 0o001, 0o007),
        ("ug=rw", 0o660, 0o770),
        ("a-rwx", 0, 0o777),
        ("o=", 0, 0o007),
        ("g+", 0, 0),
        ("u=rw,g=r,o=", 0o640, 0o777),
    ];
    for initial in 0..=0o777 {
        for (spec, add, clear) in specs {
            let mut t = tree();
            t.nodes.get_mut("/work/source").unwrap().mode = initial;
            t.execute(&format!("chmod {spec} source")).unwrap();
            assert_eq!(
                t.nodes["/work/source"].mode,
                (initial & !clear) | add,
                "{initial:03o}/{spec}"
            );
        }
        let mut t = tree();
        t.execute(&format!("chmod {initial:04o} source")).unwrap();
        assert_eq!(t.nodes["/work/source"].mode, initial);
    }
    let mut t = tree();
    t.execute("umask 777").unwrap();
    t.execute("chmod a=rw,u+x,g-w,o-r source").unwrap();
    assert_eq!(t.nodes["/work/source"].mode, 0o742);
    assert_eq!(t.files["/work/source"], "original\n");
}
#[test]
fn ownership_search_and_membership_are_enforced_atomically_for_attribute_changes() {
    let mut t = tree();
    assert!(t.valid());
    for cmd in [
        "chmod 666 foreign",
        "chgrp staff foreign",
        "chmod 600 locked/note",
        "chgrp review locked/note",
        "chgrp visitors source",
        "chgrp unknown source",
        "chgrp 2000 source",
    ] {
        let before = t.clone();
        assert!(t.execute(cmd).is_err(), "{cmd}");
        assert_eq!(t, before);
    }
    t.execute("chmod u=x locked").unwrap();
    t.execute("chmod 640 locked/note").unwrap();
    t.execute("chgrp review source").unwrap();
    assert_eq!(t.nodes["/work/source"].group, "review");
    assert_eq!(t.nodes["/work/source"].mode, 0o600);
    // chmod does not need the target's own read/write bits, only ownership and parent search.
    t.execute("chmod 000 source").unwrap();
    t.execute("chmod u=rw source").unwrap();
    assert_eq!(t.nodes["/work/source"].mode, 0o600);
    for bad in [
        "chmod +x source",
        "chmod g=u source",
        "chmod a+X source",
        "chmod 4755 source",
        "chmod -R source",
        "chmod u=rw,,g=r source",
        "chmod u+r-w source",
        "chmod 00000 source",
        "chmod u+r source foreign",
        "chown blair source",
    ] {
        let before = t.clone();
        assert!(t.execute(bad).is_err(), "{bad}");
        assert_eq!(t, before);
    }
    let too_many = format!("chmod {} source", vec!["u+r"; 33].join(","));
    let before = t.clone();
    assert!(t.execute(&too_many).unwrap_err().contains("exceeds"));
    assert_eq!(t, before);
}
#[test]
fn access_uses_owner_precedence_group_membership_and_every_parent_search_permission() {
    let mut t = tree();
    for mode in 0..=0o777 {
        t.nodes.get_mut("/work/source").unwrap().mode = mode;
        for (user, shift) in [("alex", 6), ("blair", 3), ("casey", 0)] {
            assert_eq!(
                t.access(user, "/work/source", Access::ReadFile),
                (mode >> shift) & 4 != 0,
                "{user}/{mode:03o}"
            );
            assert_eq!(
                t.access(user, "/work/source", Access::WriteFile),
                (mode >> shift) & 2 != 0
            );
        }
    }
    t.nodes.get_mut("/work/source").unwrap().mode = 0o066;
    assert!(!t.access("alex", "/work/source", Access::ReadFile));
    assert!(t.access("blair", "/work/source", Access::ReadFile));
    t.nodes.get_mut("/work/dir").unwrap().mode = 0o740;
    assert!(t.access("blair", "/work/dir", Access::ListNames));
    assert!(!t.access("blair", "/work/dir", Access::SearchDirectory));
    assert!(!t.access("blair", "/work/dir/note", Access::ReadFile));
    t.nodes.get_mut("/work/dir").unwrap().mode = 0o710;
    assert!(!t.access("blair", "/work/dir", Access::ListNames));
    assert!(t.access("blair", "/work/dir/note", Access::ReadFile));
    t.nodes.get_mut("/work/dir").unwrap().mode = 0o720;
    assert!(!t.access("blair", "/work/dir", Access::CreateEntry));
    t.nodes.get_mut("/work/dir").unwrap().mode = 0o730;
    assert!(t.access("blair", "/work/dir", Access::CreateEntry));
    t.nodes.get_mut("/").unwrap().mode = 0o740;
    assert!(!t.access("alex", "/work/source", Access::WriteFile));
    assert!(!t.access("root", "/work/source", Access::ReadFile));
}
#[test]
fn id_is_informational_and_option_termination_and_names_do_not_query_host_accounts() {
    let mut t = tree();
    let before = t.clone();
    assert_eq!(
        t.execute("id").unwrap(),
        "uid=1000(alex) gid=2001(staff) groups=2001(staff),2000(review)\n"
    );
    assert_eq!(t.execute("id -Gn").unwrap(), "staff review\n");
    assert_eq!(t.execute("id -nG blair").unwrap(), "staff\n");
    assert_eq!(
        t.execute("id root").unwrap(),
        "uid=0(root) gid=0(root) groups=0(root)\n"
    );
    for cmd in ["id -- -Gn", "id -u", "id another-account", "id alex blair"] {
        assert!(t.execute(cmd).is_err(), "{cmd}");
    }
    assert_eq!(t, before);
    t.users.get_mut("alex").unwrap().push("staff".into());
    assert_eq!(t.execute("id -Gn").unwrap(), "staff review\n");
    t.execute("touch -- -n").unwrap();
    t.execute("chmod -- 600 -n").unwrap();
    t.execute("chgrp -- review -n").unwrap();
    assert_eq!(t.nodes["/work/-n"].group, "review");
}
#[test]
fn typed_access_goals_require_existing_correct_targets_and_invalidate_when_broken() {
    let mut t = tree();
    let goal = Requirement::Access {
        path: "/work/source".into(),
        user: "blair".into(),
        operation: Access::ReadFile,
        allowed: true,
    };
    assert!(goal.valid_for(&t));
    assert!(!goal.check(&t).0);
    t.execute("chmod 640 source").unwrap();
    assert!(goal.check(&t).0);
    t.execute("chmod 600 source").unwrap();
    assert!(!goal.check(&t).0);
    let denied = Requirement::Access {
        path: "/work/source".into(),
        user: "blair".into(),
        operation: Access::ReadFile,
        allowed: false,
    };
    assert!(denied.check(&t).0);
    t.execute("rm source").unwrap();
    assert!(!denied.check(&t).0);
    t.execute("mkdir source").unwrap();
    assert!(!denied.check(&t).0);
    for user in ["missing", "root", ""] {
        let r = Requirement::Access {
            path: "/work/source".into(),
            user: user.into(),
            operation: Access::ReadFile,
            allowed: false,
        };
        assert!(!r.valid_for(&t));
        assert!(!r.check(&t).0);
    }
    let state = State::Filesystem { tree: t };
    let goal = Goal::Filesystem {
        requirements: vec![denied],
    };
    assert!(!engine::solved(&state, &goal));
    assert!(dlms_learning_lab::workspace::issues(&state, &goal)[0].contains("must be a file"));
}
#[test]
fn invalid_goal_subjects_are_rejected_and_old_progress_format_is_unchanged() {
    let labs = pack::bundled().unwrap();
    let old = labs
        .iter()
        .find(|s| s.id == "lpic1-file-navigation")
        .unwrap();
    let mut attempt = Attempt::new(old, 1);
    for cmd in &old.solution {
        engine::execute(&mut attempt.state, cmd).unwrap();
    }
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(old.id.clone(), attempt);
    let bytes = serde_json::to_vec(&p).unwrap();
    let restored = progress::decode(&bytes, &labs).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    let mut lesson: terminal_teaching::Lesson = old.terminal_teaching.clone().unwrap();
    lesson.outcomes.push(Requirement::Access {
        path: "/work/source".into(),
        user: "missing".into(),
        operation: Access::ReadFile,
        allowed: false,
    });
    assert!(lesson.validate().is_err());
    let mut s = old.clone();
    s.goal = Goal::Filesystem {
        requirements: vec![Requirement::Access {
            path: "/work/source".into(),
            user: "missing".into(),
            operation: Access::ReadFile,
            allowed: false,
        }],
    };
    let text = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![s],
    })
    .unwrap();
    assert!(
        pack::load(&text)
            .unwrap_err()
            .contains("filesystem objectives")
    );
}

#[test]
fn ownership_requirements_distinguish_in_place_update_move_and_recreation() {
    let mut t = tree();
    let requirement = Requirement::Ownership {
        path: "/work/foreign".into(),
        owner: "blair".into(),
        group: "staff".into(),
    };
    assert!(requirement.valid_for(&t));
    assert!(requirement.check(&t).0);
    t.nodes.get_mut("/work/foreign").unwrap().mode = 0o660;
    t.execute("cp source foreign").unwrap();
    assert!(requirement.check(&t).0);
    assert_eq!(t.files["/work/foreign"], "original\n");
    t.execute("rm foreign").unwrap();
    t.execute("cp source foreign").unwrap();
    assert!(!requirement.check(&t).0);
    let mut t = tree();
    t.execute("mv foreign renamed").unwrap();
    let r = Requirement::Ownership {
        path: "/work/renamed".into(),
        owner: "blair".into(),
        group: "staff".into(),
    };
    assert!(r.check(&t).0);
    for (owner, group) in [("missing", "staff"), ("blair", "missing")] {
        let r = Requirement::Ownership {
            path: "/work/renamed".into(),
            owner: owner.into(),
            group: group.into(),
        };
        assert!(!r.valid_for(&t));
        assert!(!r.check(&t).0);
    }
}
