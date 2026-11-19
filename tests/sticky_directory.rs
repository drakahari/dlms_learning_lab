use dlms_learning_lab::{
    engine::Node,
    filesystem::{
        Access, Tree,
        directory_policy::{CoreWorld, ErrorKind, MAX_RAW},
    },
};
use std::collections::{BTreeMap, BTreeSet};
fn node(directory: bool, mode: u16, owner: &str, group: &str) -> Node {
    Node {
        directory,
        mode,
        owner: owner.into(),
        group: group.into(),
    }
}
fn world() -> CoreWorld {
    let tree = Tree {
        nodes: BTreeMap::from([
            ("/".into(), node(true, 0o755, "root", "root")),
            ("/work".into(), node(true, 0o755, "alex", "staff")),
            ("/work/share".into(), node(true, 0o770, "alex", "team")),
            ("/work/peer".into(), node(true, 0o770, "blair", "team")),
            ("/work/public".into(), node(true, 0o777, "alex", "team")),
            (
                "/work/share/alex.txt".into(),
                node(false, 0o400, "alex", "team"),
            ),
            (
                "/work/share/blair.txt".into(),
                node(false, 0o444, "blair", "team"),
            ),
            (
                "/work/share/writable.txt".into(),
                node(false, 0o666, "blair", "team"),
            ),
            (
                "/work/peer/alex.txt".into(),
                node(false, 0o400, "alex", "team"),
            ),
            (
                "/work/peer/foreign.txt".into(),
                node(false, 0o400, "blair", "team"),
            ),
            (
                "/work/peer/replace.txt".into(),
                node(false, 0o600, "alex", "team"),
            ),
            (
                "/work/public/casey.txt".into(),
                node(false, 0o000, "casey", "team"),
            ),
        ]),
        files: BTreeMap::from([
            ("/work/share/alex.txt".into(), "Alex original\n".into()),
            ("/work/share/blair.txt".into(), "Blair original\n".into()),
            (
                "/work/share/writable.txt".into(),
                "Writable evidence\n".into(),
            ),
            ("/work/peer/alex.txt".into(), "Current publication\n".into()),
            (
                "/work/peer/foreign.txt".into(),
                "Protected foreign publication\n".into(),
            ),
            (
                "/work/peer/replace.txt".into(),
                "Own superseded publication\n".into(),
            ),
            (
                "/work/public/casey.txt".into(),
                "Mode zero is not deletion protection\n".into(),
            ),
        ]),
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into(), "team".into()]),
            ("blair".into(), vec!["team".into()]),
            ("casey".into(), vec!["team".into()]),
            ("devon".into(), vec!["visitors".into()]),
        ]),
        user: "alex".into(),
        home: "/work".into(),
        cwd: "/work".into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: None,
    };
    CoreWorld::new(
        tree,
        BTreeSet::from(["/work/share".into(), "/work/peer".into()]),
    )
    .unwrap()
}
fn rejected(w: &mut CoreWorld, input: &str) -> ErrorKind {
    let old = w.clone();
    let e = w.execute(input).unwrap_err();
    assert_eq!(*w, old, "{input}");
    e.kind
}
#[test]
fn owner_file_owner_and_peer_matrix_executes_discarded_operations_without_state_changes() {
    let w = world();
    let before = w.clone();
    for (dir, owner) in [("share", "alex"), ("peer", "blair")] {
        for leaf in if dir == "share" {
            ["alex.txt", "blair.txt"]
        } else {
            ["alex.txt", "foreign.txt"]
        } {
            let path = format!("/work/{dir}/{leaf}");
            let fileowner = w.tree.nodes[&path].owner.as_str();
            for actor in ["alex", "blair", "casey"] {
                let expected = actor == owner || actor == fileowner;
                for command in [
                    format!("rm '{path}'"),
                    format!("mv '{path}' '/work/{dir}/renamed.txt'"),
                ] {
                    let p = w.probe(actor, &command).unwrap();
                    assert_eq!(p.succeeded, expected, "{actor}: {command}");
                    assert_eq!(
                        p.failure_kind,
                        if expected {
                            None
                        } else {
                            Some(ErrorKind::Permission)
                        }
                    );
                    if !expected {
                        assert!(p.diagnostic.contains("Restricted deletion"));
                    }
                    assert_eq!(w, before);
                }
            }
        }
    }
}
#[test]
fn removing_sticky_restores_ordinary_deletion_without_file_write_permission() {
    let mut w = world();
    w.sticky.clear();
    assert!(w.probe("casey", "rm share/alex.txt").unwrap().succeeded);
    assert!(
        !w.ordinary_access("casey", "/work/share/alex.txt", Access::WriteFile)
            .unwrap()
    );
    assert!(w.probe("devon", "rm public/casey.txt").unwrap().succeeded);
    w.sticky.insert("/work/public".into());
    assert!(!w.probe("devon", "rm public/casey.txt").unwrap().succeeded);
    assert!(w.probe("casey", "rm public/casey.txt").unwrap().succeeded);
}
#[test]
fn parent_write_search_and_ancestor_search_precede_owner_exceptions_and_owner_class_wins() {
    let mut w = world();
    w.tree.nodes.get_mut("/work/peer").unwrap().mode = 0o070;
    let p = w.probe("blair", "rm peer/foreign.txt").unwrap();
    assert!(!p.succeeded);
    assert!(p.diagnostic.contains("directory search"));
    assert!(w.probe("alex", "rm peer/alex.txt").unwrap().succeeded);
    w.tree.nodes.get_mut("/work").unwrap().mode = 0o700;
    let p = w.probe("blair", "rm peer/foreign.txt").unwrap();
    assert!(!p.succeeded);
    assert!(!p.diagnostic.contains("Restricted deletion"));
    w.tree.nodes.get_mut("/work").unwrap().mode = 0o755;
    w.tree.nodes.get_mut("/work/share").unwrap().mode = 0o550;
    assert!(!w.probe("alex", "rm share/blair.txt").unwrap().succeeded);
    w.tree.nodes.get_mut("/work/share").unwrap().mode = 0o760;
    assert!(!w.probe("blair", "rm share/blair.txt").unwrap().succeeded);
}
#[test]
fn collaborative_creation_and_content_access_are_not_sticky_authorization() {
    let w = world();
    assert!(
        w.ordinary_access("blair", "/work/share", Access::CreateEntry)
            .unwrap()
    );
    assert!(
        !w.ordinary_access("devon", "/work/share", Access::CreateEntry)
            .unwrap()
    );
    assert!(
        w.ordinary_access("casey", "/work/share/writable.txt", Access::WriteFile)
            .unwrap()
    );
    assert!(!w.probe("casey", "rm share/writable.txt").unwrap().succeeded);
    assert_eq!(
        w.tree.files["/work/share/writable.txt"],
        "Writable evidence\n"
    );
}
#[test]
fn chmod_add_remove_numeric_clear_symbolic_preserve_and_t_uppercase_t() {
    let mut w = world();
    w.execute("chmod 0777 share").unwrap();
    assert!(!w.sticky.contains("/work/share"));
    w.execute("chmod 01777 share").unwrap();
    assert_eq!(w.effective_mode("/work/share").unwrap(), 0o1777);
    w.execute("chmod -t share").unwrap();
    w.execute("chmod +t share").unwrap();
    assert!(
        w.execute("ls -ld share")
            .unwrap()
            .starts_with("drwxrwxrwt ")
    );
    w.execute("chmod o-x share").unwrap();
    assert_eq!(w.effective_mode("/work/share").unwrap(), 0o1776);
    assert!(
        w.execute("ls -ld share")
            .unwrap()
            .starts_with("drwxrwxrwT ")
    );
    w.execute("chmod -t share").unwrap();
    assert_eq!(w.effective_mode("/work/share").unwrap(), 0o776);
    w.execute("chmod 1770 share").unwrap();
    assert_eq!(w.effective_mode("/work/share").unwrap(), 0o1770);
    w.execute("chmod g=rx share").unwrap();
    assert_eq!(w.effective_mode("/work/share").unwrap(), 0o1750);
    w.execute("chmod 0770 share").unwrap();
    assert_eq!(w.effective_mode("/work/share").unwrap(), 0o770);
    assert_eq!(rejected(&mut w, "chmod +t peer"), ErrorKind::Permission);
}
#[test]
fn source_sticky_denies_before_rename_publication_and_preserves_every_field() {
    let mut w = world();
    let before = w.clone();
    let e = w
        .execute("mv peer/foreign.txt share/fresh.txt")
        .unwrap_err();
    assert_eq!(e.kind, ErrorKind::Permission);
    assert!(e.message.contains("peer/foreign.txt"));
    assert_eq!(w, before);
}
#[test]
fn destination_replacement_has_independent_sticky_and_parent_checks() {
    let mut w = world();
    let e = w.execute("mv share/alex.txt peer/foreign.txt").unwrap_err();
    assert_eq!(e.kind, ErrorKind::Permission);
    assert!(e.message.contains("peer/foreign.txt"));
    assert!(w.tree.nodes.contains_key("/work/share/alex.txt"));
    assert_eq!(
        w.tree.files["/work/peer/foreign.txt"],
        "Protected foreign publication\n"
    );
    w.tree.nodes.get_mut("/work/peer").unwrap().mode = 0o550;
    let before = w.clone();
    let e = w.execute("mv share/alex.txt peer/foreign.txt").unwrap_err();
    assert!(e.message.contains("write and search"));
    assert_eq!(w, before);
}
#[test]
fn fresh_destination_and_owned_replacement_preserve_source_metadata_and_other_records() {
    for destination in ["peer/fresh.txt", "peer/replace.txt"] {
        let mut w = world();
        let node = w.tree.nodes["/work/share/alex.txt"].clone();
        let foreign = w.tree.files["/work/peer/foreign.txt"].clone();
        w.execute(&format!("mv share/alex.txt {destination}"))
            .unwrap();
        assert!(!w.tree.nodes.contains_key("/work/share/alex.txt"));
        assert_eq!(w.tree.nodes[&format!("/work/{destination}")], node);
        assert_eq!(
            w.tree.files[&format!("/work/{destination}")],
            "Alex original\n"
        );
        assert_eq!(w.tree.files["/work/peer/foreign.txt"], foreign);
    }
    let mut w = world();
    w.execute("mv peer/alex.txt share/blair.txt").unwrap();
    assert_eq!(
        w.tree.files["/work/share/blair.txt"],
        "Current publication\n"
    ); // destination-directory owner exception
}
#[test]
fn quoted_literal_paths_directory_operands_and_end_of_options_keep_the_same_checks() {
    let mut w = world();
    w.tree.nodes.insert(
        "/work/share/-a file".into(),
        node(false, 0o400, "alex", "team"),
    );
    w.tree
        .files
        .insert("/work/share/-a file".into(), "Literal name\n".into());
    w.execute("mv -- 'share/-a file' 'share/next item'")
        .unwrap();
    assert_eq!(w.tree.files["/work/share/next item"], "Literal name\n");
    w.execute("rm -- 'share/next item'").unwrap();
    w.execute("mv peer/alex.txt share").unwrap();
    assert_eq!(
        w.tree.files["/work/share/alex.txt"],
        "Current publication\n"
    );
    w.execute("rm -f -- no-such-file").unwrap();
}
#[test]
fn unsupported_syntax_privilege_special_modes_and_directory_mutations_are_atomic() {
    let mut w = world();
    for input in [
        "chmod 4755 share",
        "chmod 2770 share",
        "chmod 7777 share",
        "chmod o=t share",
        "chmod u+t share",
        "chmod g+t share",
        "chmod +t share/alex.txt",
        "chmod 1777 share/alex.txt",
        "chmod +t,-t share",
        "chmod -R share",
        "rm -rf share",
        "rmdir share",
        "mv share peer/new",
        "cp -p share/alex.txt peer/copy.txt",
        "touch -r share/alex.txt",
        "cat share/alex.txt > peer/new",
        "pwd -bad",
        "cat -bad",
        "id -bad",
        "ls -i share",
    ] {
        let kind = rejected(&mut w, input);
        assert_eq!(kind, ErrorKind::Unsupported, "{input}");
    }
    assert_eq!(
        w.probe("root", "rm share/alex.txt").unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        w.probe("nobody", "rm share/alex.txt").unwrap_err().kind,
        ErrorKind::Invalid
    );
    assert_eq!(
        w.probe("blair", "cat share/alex.txt").unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        w.ordinary_access("root", "/work/share", Access::CreateEntry)
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
}
#[test]
fn bounded_world_decode_duplicate_paths_output_and_clock_fail_without_partial_publication() {
    let mut w = world();
    w.tree
        .files
        .insert("/work/share/alex.txt".into(), "x".repeat(32_769));
    assert!(w.validate().is_ok());
    assert_eq!(rejected(&mut w, "cat share/alex.txt"), ErrorKind::Resource);
    let mut w = world();
    w.tree.clock = 1_000_000;
    assert_eq!(rejected(&mut w, "chmod -t share"), ErrorKind::Resource);
    let mut w = world();
    assert_eq!(rejected(&mut w, "chmod +t public"), ErrorKind::Resource);
    let mut bad = world();
    bad.sticky = BTreeSet::from(["/work/share/alex.txt".into()]);
    assert!(bad.validate().is_err());
    let mut bad = world();
    bad.tree.nodes.get_mut("/work/share").unwrap().mode = 0o1770;
    assert!(bad.validate().is_err());
    let mut value = serde_json::to_value(world()).unwrap();
    value["sticky"] = serde_json::json!(["/work/share", "/work/share"]);
    assert!(CoreWorld::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    assert!(CoreWorld::decode(&vec![b' '; MAX_RAW + 1]).is_err());
    assert_eq!(
        rejected(&mut world(), &"x".repeat(4_097)),
        ErrorKind::Resource
    );
}
#[test]
fn readonly_formats_keep_plain_listing_and_data_and_never_expand_or_run_files() {
    let mut w = world();
    let old = w.clone();
    assert_eq!(w.execute("pwd").unwrap(), "/work\n");
    assert!(w.execute("id -Gn blair").unwrap().contains("team"));
    assert_eq!(w.execute("cat share/alex.txt").unwrap(), "Alex original\n");
    let mut t = w.tree.clone();
    assert_eq!(
        w.execute("ls share").unwrap(),
        t.execute("ls share").unwrap()
    );
    assert_eq!(w, old);
    assert_eq!(
        rejected(&mut w, "$(rm share/alex.txt)"),
        ErrorKind::Unsupported
    );
    assert_eq!(
        rejected(&mut w, "rm share/alex.txt; id"),
        ErrorKind::Unsupported
    );
}
#[test]
fn old_tree_wire_and_ordinary_mode_deletion_behavior_remain_unchanged() {
    let w = world();
    let raw = serde_json::to_vec(&w.tree).unwrap();
    let decoded: Tree = serde_json::from_slice(&raw).unwrap();
    assert_eq!(decoded, w.tree);
    let value = serde_json::to_value(&w.tree).unwrap();
    assert!(value.get("sticky").is_none());
    assert!(value.get("command_location").is_none());
    let mut old = w.tree.clone();
    old.user = "casey".into();
    old.execute("rm share/blair.txt").unwrap();
    assert!(!w.probe("casey", "rm share/blair.txt").unwrap().succeeded);
    let before = old.clone();
    assert!(old.execute("chmod +t share").is_err());
    assert_eq!(old, before);
    assert_eq!(
        CoreWorld::decode(&serde_json::to_vec(&w).unwrap()).unwrap(),
        w
    );
}

#[test]
fn malformed_authority_free_worlds_and_foreign_context_cannot_enter_the_core() {
    let mut bad = world();
    bad.tree.user = "root".into();
    assert!(bad.validate().is_err());
    let mut bad = world();
    bad.tree.command_location = Some(dlms_learning_lab::command_location::Context {
        version: 1,
        source_id: "other-activity".into(),
        path: "/work".into(),
        names: vec!["worker".into()],
    });
    assert!(bad.validate().is_err());
    let mut bad = world();
    bad.sticky = BTreeSet::from(["/work/missing".into()]);
    assert!(bad.validate().is_err());
    let mut bad = world();
    bad.sticky = BTreeSet::from(["/work/../work/share".into()]);
    assert!(bad.validate().is_err());
    let mut bad = world();
    for n in 0..65 {
        let p = format!("/work/n{n}");
        bad.tree
            .nodes
            .insert(p.clone(), node(false, 0o600, "alex", "staff"));
        bad.tree.files.insert(p, String::new());
    }
    assert!(bad.validate().is_err());
    let mut bad = world();
    bad.tree
        .files
        .insert("/work/share/alex.txt".into(), "x".repeat(131_073));
    assert!(bad.validate().is_err());
    let mut bad = world();
    bad.tree.users.insert("extra".into(), vec!["staff".into()]);
    assert!(bad.validate().is_err());
    let mut value = serde_json::to_value(world()).unwrap();
    value["extra"] = serde_json::json!(true);
    assert!(CoreWorld::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    // No source binding is asserted by this generic world. Future policy must
    // reject changed-but-structurally-valid users, owners, sticky flags or slots.
}

#[test]
fn actual_participant_creation_and_removal_keep_other_users_files_and_discard_probes() {
    let w = world();
    let before = w.clone();
    assert!(w.probe("blair", "touch share/new.txt").unwrap().succeeded);
    assert!(!w.probe("devon", "touch share/new.txt").unwrap().succeeded);
    assert_eq!(w, before);
    let mut participant = w.clone();
    participant.tree.user = "blair".into();
    participant.execute("touch share/new.txt").unwrap();
    assert_eq!(participant.tree.files["/work/share/new.txt"], "");
    assert_eq!(
        participant.tree.nodes["/work/share/new.txt"],
        node(false, 0o644, "blair", "team")
    );
    assert!(participant.sticky.contains("/work/share"));
    participant.execute("rm share/new.txt").unwrap();
    assert!(!participant.tree.nodes.contains_key("/work/share/new.txt"));
    assert_eq!(participant.tree.files, w.tree.files);
    assert_eq!(participant.tree.nodes, w.tree.nodes);
    let old = participant.clone();
    assert_eq!(
        rejected(&mut participant, "touch share"),
        ErrorKind::Unsupported
    );
    assert_eq!(participant, old);
}
#[test]
fn actual_copy_overwrites_writable_contents_without_sticky_bypass_or_owner_change() {
    let mut w = world();
    w.tree.nodes.get_mut("/work/share/alex.txt").unwrap().mode = 0o644;
    let before = w.clone();
    assert!(
        w.probe("casey", "cp share/alex.txt share/writable.txt")
            .unwrap()
            .succeeded
    );
    assert!(!w.probe("casey", "rm share/writable.txt").unwrap().succeeded);
    assert_eq!(w, before);
    w.tree.user = "casey".into();
    let target = w.tree.nodes["/work/share/writable.txt"].clone();
    w.execute("cp share/alex.txt share/writable.txt").unwrap();
    assert_eq!(w.tree.files["/work/share/writable.txt"], "Alex original\n");
    assert_eq!(w.tree.nodes["/work/share/writable.txt"], target);
    assert_eq!(
        rejected(&mut w, "rm share/writable.txt"),
        ErrorKind::Permission
    );
    assert_eq!(
        rejected(&mut w, "cp share/alex.txt peer/foreign.txt"),
        ErrorKind::Permission
    );
    w.execute("cp share/alex.txt share/copy.txt").unwrap();
    assert_eq!(
        w.tree.nodes["/work/share/copy.txt"],
        node(false, 0o644, "casey", "team")
    );
    assert_eq!(w.tree.files["/work/share/copy.txt"], "Alex original\n");
    w.tree.nodes.get_mut("/work/share").unwrap().mode = 0o550;
    // Existing writable contents remain writable with directory search but no
    // directory write, unlike creation/removal of a directory entry.
    w.execute("cp share/alex.txt share/writable.txt").unwrap();
    assert_eq!(
        rejected(&mut w, "cp share/alex.txt share/new-copy.txt"),
        ErrorKind::Permission
    );
}

#[test]
fn create_and_copy_resource_preflight_remains_atomic() {
    let mut w = world();
    w.tree
        .files
        .insert("/work/share/alex.txt".into(), "x".repeat(100_000));
    assert!(w.validate().is_ok());
    assert_eq!(
        rejected(&mut w, "cp share/alex.txt share/new.txt"),
        ErrorKind::Resource
    );
    let mut w = world();
    w.tree.clock = 1_000_000;
    assert_eq!(rejected(&mut w, "touch share/new.txt"), ErrorKind::Resource);
    assert_eq!(
        rejected(&mut w, "cp share/alex.txt share/writable.txt"),
        ErrorKind::Resource
    );
}

#[test]
fn symbolic_equals_selecting_other_clears_sticky_in_order_while_u_g_and_plus_minus_preserve() {
    for (spec, mode, sticky) in [
        ("a=rwx", 0o777, false),
        ("o=rx", 0o775, false),
        ("o=", 0o770, false),
        ("u=rw", 0o670, true),
        ("g=rx", 0o750, true),
        ("ugo=rw", 0o666, false),
        ("u=rw,g=rx", 0o650, true),
        ("u=rw,o=rx,g=rx", 0o655, false),
        ("o=,u=rwx,g=rw", 0o760, false),
        ("a=rw,u+x,o-r", 0o762, false),
        ("u+x,g-w,o+r", 0o754, true),
        ("o-r,u=rw,g+x", 0o670, true),
    ] {
        let mut w = world();
        w.execute(&format!("chmod {spec} share")).unwrap();
        assert_eq!(w.tree.nodes["/work/share"].mode, mode, "{spec}");
        assert_eq!(w.sticky.contains("/work/share"), sticky, "{spec}");
    }
    for numeric in ["777", "0777", "00000"] {
        let mut w = world();
        w.execute(&format!("chmod {numeric} share")).unwrap();
        assert!(!w.sticky.contains("/work/share"), "{numeric}");
    }
}
#[test]
fn ordinary_actor_count_is_four_even_without_root_metadata() {
    let mut w = world();
    for node in w
        .tree
        .nodes
        .values_mut()
        .filter(|node| node.owner == "root")
    {
        node.owner = "alex".into();
        node.group = "staff".into();
    }
    w.tree.users.remove("root");
    assert!(w.validate().is_ok());
    w.tree.users.insert("fifth".into(), vec!["staff".into()]);
    assert_eq!(w.tree.users.len(), 5);
    assert_eq!(w.validate().unwrap_err().kind, ErrorKind::Resource);
    w.tree.users.remove("fifth");
    w.tree.users.insert("root".into(), vec!["root".into()]);
    assert!(w.validate().is_ok());
}
#[test]
fn probe_failure_kind_distinguishes_denial_missing_and_operation_and_never_masks_invalid_source() {
    let w = world();
    assert_eq!(
        w.probe("casey", "rm share/blair.txt").unwrap().failure_kind,
        Some(ErrorKind::Permission)
    );
    assert_eq!(
        w.probe("casey", "rm share/missing.txt")
            .unwrap()
            .failure_kind,
        Some(ErrorKind::Missing)
    );
    assert_eq!(
        w.probe("alex", "mv share/alex.txt share/alex.txt")
            .unwrap()
            .failure_kind,
        Some(ErrorKind::Operation)
    );
    let p = w.probe("blair", "touch share/new.txt").unwrap();
    assert!(p.succeeded);
    assert_eq!(p.failure_kind, None);
    assert_eq!(
        w.probe("nobody", "rm share/blair.txt").unwrap_err().kind,
        ErrorKind::Invalid
    );
    assert_eq!(
        w.probe("casey", "rm -rf share").unwrap_err().kind,
        ErrorKind::Unsupported
    );
    let mut bad = w.clone();
    bad.tree.command_location = Some(dlms_learning_lab::command_location::Context {
        version: 1,
        source_id: "foreign".into(),
        path: "/work".into(),
        names: vec!["worker".into()],
    });
    assert_eq!(
        bad.probe("casey", "rm share/blair.txt").unwrap_err().kind,
        ErrorKind::Invalid
    );
    // Generic core shape is not original-source authority; later Definition
    // must separately refuse structurally valid owner/role/slot forgeries.
}
