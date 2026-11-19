//! Fixed in-process private-index tests; no host utilities or learner files.
use dlms_learning_lab::{
    engine::Node,
    file_index::{self, Context, ErrorKind, Status},
    filesystem::Tree,
};
use std::collections::BTreeMap;
fn node(dir: bool, mode: u16, owner: &str) -> Node {
    Node {
        directory: dir,
        mode,
        owner: owner.into(),
        group: if owner == "root" { "root" } else { "staff" }.into(),
    }
}
fn fixture() -> (Context, Tree) {
    let mut nodes = BTreeMap::new();
    for p in [
        "/",
        "/work",
        "/docs",
        "/docs/current",
        "/docs/archive",
        "/docs/archives",
        "/docs/private",
        "/docs/locked",
    ] {
        nodes.insert(
            p.into(),
            node(true, 0o755, if p == "/" { "root" } else { "alex" }),
        );
    }
    let mut files = BTreeMap::new();
    for (p, text) in [
        (
            "/work/index.conf",
            "PRUNEPATHS = \"/docs/archive /docs/private\"\n",
        ),
        ("/docs/current/release.txt", "current release\n"),
        ("/docs/archive/release.txt", "old archive\n"),
        ("/docs/archives/retained.txt", "not the pruned directory\n"),
        ("/docs/private/note.txt", "private content remains\n"),
        ("/docs/locked/item.txt", "known visible item\n"),
    ] {
        nodes.insert(p.into(), node(false, 0o644, "alex"));
        files.insert(p.into(), text.into());
    }
    let tree = Tree {
        nodes,
        files,
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into()]),
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
    let c = Context {
        version: 1,
        source_id: "index-assignment".into(),
        root: "/docs".into(),
        database: "/work/catalog.db".into(),
        config: "/work/index.conf".into(),
    };
    assert!(tree.valid());
    c.validate_source(&tree).unwrap();
    (c, tree)
}
fn build(c: &Context, t: &Tree) -> file_index::Index {
    file_index::rebuild(c, Some(c), t, t).unwrap()
}
fn command(c: &Context) -> String {
    format!(
        "updatedb -l 0 -o {} -U {} --config-file {}",
        c.database, c.root, c.config
    )
}
#[test]
fn rebuild_retains_pruned_directory_not_descendants_and_prefix_is_safe() {
    let (c, t) = fixture();
    let before = t.clone();
    let i = build(&c, &t);
    assert!(
        !i.entries().contains(&"/docs".into()),
        "The scan root is not a query result"
    );
    assert!(i.entries().contains(&"/docs/archive".into()));
    assert!(!i.entries().contains(&"/docs/archive/release.txt".into()));
    assert!(i.entries().contains(&"/docs/archives/retained.txt".into()));
    assert!(!i.entries().contains(&"/docs/private/note.txt".into()));
    assert_eq!(t, before);
    assert!(file_index::matches_current(&i, &c, Some(&c), &t, &t).unwrap());
}
#[test]
fn names_do_not_require_file_read_permission() {
    let (c, mut t) = fixture();
    t.nodes.get_mut("/docs/current/release.txt").unwrap().mode = 0;
    let i = build(&c, &t);
    let o = file_index::locate(&i, &c, Some(&c), &t, &t, "release.txt", false).unwrap();
    assert_eq!(o.status, Status::Found);
    assert_eq!(o.paths, ["/docs/current/release.txt"]);
}
#[test]
fn inaccessible_pruned_contents_are_not_scanned() {
    let (c, mut t) = fixture();
    t.nodes.get_mut("/docs/private").unwrap().mode = 0;
    let i = build(&c, &t);
    assert!(i.entries().contains(&"/docs/private".into()));
    assert!(!i.entries().iter().any(|p| p.starts_with("/docs/private/")));
}
#[test]
fn stale_removed_name_and_new_live_name_differ_until_rebuild() {
    let (c, source) = fixture();
    let i = build(&c, &source);
    let mut current = source.clone();
    current.execute("rm /docs/current/release.txt").unwrap();
    current.execute("touch /docs/current/future.txt").unwrap();
    assert_eq!(
        file_index::locate(&i, &c, Some(&c), &source, &current, "release.txt", false)
            .unwrap()
            .paths,
        ["/docs/current/release.txt"]
    );
    assert_eq!(
        file_index::locate(&i, &c, Some(&c), &source, &current, "release.txt", true)
            .unwrap()
            .status,
        Status::NoMatch
    );
    assert_eq!(
        file_index::locate(&i, &c, Some(&c), &source, &current, "future.txt", false)
            .unwrap()
            .status,
        Status::NoMatch
    );
    assert!(!file_index::matches_current(&i, &c, Some(&c), &source, &current).unwrap());
    let next = file_index::rebuild(&c, Some(&c), &source, &current).unwrap();
    assert_eq!(
        file_index::locate(&next, &c, Some(&c), &source, &current, "future.txt", false)
            .unwrap()
            .paths,
        ["/docs/current/future.txt"]
    );
}
#[test]
fn configuration_edit_is_not_a_rebuild_or_a_saved_observation() {
    let (c, source) = fixture();
    let i = build(&c, &source);
    let mut current = source.clone();
    current
        .files
        .insert(c.config.clone(), "PRUNEPATHS = \"/docs/archive\"\n".into());
    assert_eq!(
        file_index::locate(&i, &c, Some(&c), &source, &current, "note.txt", false)
            .unwrap()
            .status,
        Status::NoMatch
    );
    assert!(!file_index::matches_current(&i, &c, Some(&c), &source, &current).unwrap());
    let next = file_index::rebuild(&c, Some(&c), &source, &current).unwrap();
    assert_eq!(
        file_index::locate(&next, &c, Some(&c), &source, &current, "note.txt", false)
            .unwrap()
            .paths,
        ["/docs/private/note.txt"]
    );
}
#[test]
fn equivalent_config_spacing_order_duplicates_and_rebuilds_are_semantic() {
    let (c, source) = fixture();
    let i = build(&c, &source);
    let mut current = source.clone();
    current.files.insert(c.config.clone(),"# comment\n PRUNEPATHS=\"/docs/private /docs/archive /docs/private /docs/archive/old\" # end\n".into());
    assert!(file_index::matches_current(&i, &c, Some(&c), &source, &current).unwrap());
    assert_eq!(
        file_index::rebuild(&c, Some(&c), &source, &current).unwrap(),
        i
    );
}
#[test]
fn incomplete_read_or_search_does_not_publish_over_previous_index() {
    let (c, source) = fixture();
    let i = build(&c, &source);
    for mode in [0o111, 0o444, 0o700] {
        let mut current = source.clone();
        let n = current.nodes.get_mut("/docs/locked").unwrap();
        n.mode = mode;
        if mode == 0o700 {
            n.owner = "root".into();
            n.group = "root".into();
        }
        let mut saved = Some(i.clone());
        let before = saved.clone();
        let e =
            file_index::execute_bound(&c, Some(&c), &source, &current, &mut saved, &command(&c))
                .unwrap_err();
        assert_eq!(e.kind, ErrorKind::Incomplete);
        assert_eq!(saved, before);
    }
}
#[test]
fn invalid_config_and_wrong_database_are_atomic() {
    let (c, source) = fixture();
    let mut current = source.clone();
    current
        .files
        .insert(c.config.clone(), "PRUNEPATHS=\"/docs/archive\n".into());
    let mut saved = Some(build(&c, &source));
    let before = saved.clone();
    assert_eq!(
        file_index::execute_bound(&c, Some(&c), &source, &current, &mut saved, &command(&c))
            .unwrap_err()
            .kind,
        ErrorKind::Invalid
    );
    assert_eq!(saved, before);
    assert_eq!(
        file_index::execute_bound(
            &c,
            Some(&c),
            &source,
            &source,
            &mut saved,
            "locate -d /work/other.db release"
        )
        .unwrap_err()
        .kind,
        ErrorKind::SourceMismatch
    );
    assert_eq!(saved, before);
}
#[test]
fn explicit_private_commands_parse_without_shell_or_selector_escape() {
    let (c, t) = fixture();
    let mut saved = None;
    assert_eq!(
        file_index::execute_bound(
            &c,
            Some(&c),
            &t,
            &t,
            &mut saved,
            "locate -d /work/catalog.db release"
        )
        .unwrap_err()
        .kind,
        ErrorKind::MissingDatabase
    );
    assert_eq!(
        file_index::execute_bound(&c, Some(&c), &t, &t, &mut saved, &command(&c))
            .unwrap()
            .status,
        Status::Rebuilt
    );
    let o = file_index::execute_bound(
        &c,
        Some(&c),
        &t,
        &t,
        &mut saved,
        "locate -e -d /work/catalog.db release",
    )
    .unwrap();
    assert_eq!(o.stdout, "/docs/current/release.txt\n");
    assert_eq!(
        file_index::parse_command(
            &c,
            "updatedb --config-file /work/index.conf -U /docs -o /work/catalog.db -l 0"
        )
        .unwrap(),
        file_index::Command::Updatedb
    );
    assert_eq!(
        file_index::parse_command(&c, "locate -d /work/catalog.db -e release").unwrap(),
        file_index::Command::Locate {
            literal: "release".into(),
            existing: true
        }
    );
}
#[test]
fn unsupported_options_patterns_and_shell_composition_never_mutate() {
    let (c, t) = fixture();
    let mut saved = Some(build(&c, &t));
    let before = saved.clone();
    for cmd in [
        "locate release",
        "locate -d /work/catalog.db '*.txt'",
        "locate -d /work/catalog.db a b",
        "locate -r -d /work/catalog.db release",
        "locate -e -e -d /work/catalog.db release",
        "locate -d /work/catalog.db release; true",
        "locate -d /work/catalog.db $(printf release)",
        "locate -d /work/catalog.db release > out",
        "updatedb",
        "updatedb -l 1 -o /work/catalog.db -U /docs --config-file /work/index.conf",
        "updatedb -l 0 -o /work/catalog.db -U /docs --config-file /work/index.conf --debug-pruning",
    ] {
        assert!(
            file_index::execute_bound(&c, Some(&c), &t, &t, &mut saved, cmd).is_err(),
            "{cmd}"
        );
        assert_eq!(saved, before);
    }
}
#[test]
fn held_source_roles_and_identity_are_required_on_read_and_rebuild() {
    let (c, t) = fixture();
    let i = build(&c, &t);
    let mut forged = c.clone();
    forged.source_id = "other-example".into();
    assert_eq!(
        file_index::rebuild(&c, None, &t, &t).unwrap_err().kind,
        ErrorKind::SourceMismatch
    );
    assert_eq!(
        file_index::rebuild(&c, Some(&forged), &t, &t)
            .unwrap_err()
            .kind,
        ErrorKind::SourceMismatch
    );
    assert_eq!(
        file_index::locate(&i, &forged, Some(&forged), &t, &t, "release", false)
            .unwrap_err()
            .kind,
        ErrorKind::SourceMismatch
    );
    let mut current = t.clone();
    current.users.insert("blair".into(), vec!["staff".into()]);
    assert_eq!(
        file_index::rebuild(&c, Some(&c), &t, &current)
            .unwrap_err()
            .kind,
        ErrorKind::SourceMismatch
    );
}
#[test]
fn command_location_and_role_collision_are_refused() {
    let (c, mut t) = fixture();
    t.command_location = Some(dlms_learning_lab::command_location::Context {
        version: 1,
        source_id: "foreign".into(),
        path: "/docs".into(),
        names: vec!["report".into()],
    });
    assert_eq!(
        c.validate_source(&t).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    let (c, t) = fixture();
    let mut wrong = c.clone();
    wrong.database = wrong.config.clone();
    assert_eq!(wrong.shape().unwrap_err().kind, ErrorKind::Unsupported);
    let mut wrong = c.clone();
    wrong.database = "/docs/catalog.db".into();
    assert_eq!(wrong.shape().unwrap_err().kind, ErrorKind::Unsupported);
    let mut current = t.clone();
    current
        .nodes
        .insert(c.database.clone(), node(false, 0o600, "alex"));
    current
        .files
        .insert(c.database.clone(), "protected unrelated output".into());
    assert_eq!(
        file_index::rebuild(&c, Some(&c), &t, &current)
            .unwrap_err()
            .kind,
        ErrorKind::SourceMismatch
    );
}
#[test]
fn configuration_grammar_and_prefix_rules_are_deliberately_narrow() {
    for raw in [
        "",
        "PRUNEFS=\"tmpfs\"",
        "PRUNEPATHS='/docs/archive'",
        "PRUNEPATHS=\"/docs/../other\"",
        "PRUNEPATHS=\"/docs\"",
        "PRUNEPATHS=\"/elsewhere\"",
        "PRUNEPATHS=\"/docs/archive\"\nPRUNEPATHS=\"\"",
        "PRUNEPATHS=\"/docs/a\\ b\"",
        "PRUNEPATHS=\"/docs/archive\" trailing",
    ] {
        assert!(file_index::parse_config(raw, "/docs").is_err(), "{raw}");
    }
    assert!(
        file_index::parse_config("PRUNEPATHS=\"\"", "/docs")
            .unwrap()
            .directories()
            .is_empty()
    );
    let (c, mut t) = fixture();
    t.files.insert(
        c.config.clone(),
        "PRUNEPATHS=\"/docs/current/release.txt\"".into(),
    );
    assert_eq!(
        file_index::rebuild(&c, Some(&c), &t, &t).unwrap_err().kind,
        ErrorKind::Unsupported
    );
}
#[test]
fn raw_and_typed_index_admission_does_not_treat_import_as_freshness() {
    let (c, t) = fixture();
    let i = build(&c, &t);
    let bytes = serde_json::to_vec(&i).unwrap();
    assert_eq!(file_index::Index::decode(&bytes, &c, &t).unwrap(), i);
    let mut v = serde_json::to_value(&i).unwrap();
    v["entries"] = serde_json::json!(["/docs/fiction.txt"]);
    let parsed = file_index::Index::decode(&serde_json::to_vec(&v).unwrap(), &c, &t).unwrap();
    assert!(!file_index::matches_current(&parsed, &c, Some(&c), &t, &t).unwrap());
    v["entries"] = serde_json::json!(["/etc/private"]);
    assert!(file_index::Index::decode(&serde_json::to_vec(&v).unwrap(), &c, &t).is_err());
    v["entries"] = serde_json::json!(["/docs/fiction.txt", "/docs/fiction.txt"]);
    assert!(file_index::Index::decode(&serde_json::to_vec(&v).unwrap(), &c, &t).is_err());
    v["entries"] = serde_json::Value::Array(
        (0..65)
            .map(|n| serde_json::json!(format!("/docs/a{n:02}")))
            .collect(),
    );
    assert!(file_index::Index::decode(&serde_json::to_vec(&v).unwrap(), &c, &t).is_err());
    assert_eq!(
        file_index::Index::decode(&vec![b' '; file_index::MAX_INDEX_JSON + 1], &c, &t)
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
}
#[test]
fn malformed_context_duplicate_fields_and_oversized_data_are_refused() {
    let (c, t) = fixture();
    let bytes = serde_json::to_vec(&c).unwrap();
    assert_eq!(Context::decode(&bytes).unwrap(), c);
    assert!(Context::decode(br#"{"version":1,"version":1,"source_id":"x","root":"/docs","database":"/work/db","config":"/work/index.conf"}"#).is_err());
    assert_eq!(
        Context::decode(&vec![b' '; file_index::MAX_CONTEXT_JSON + 1])
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    let mut many = t.clone();
    for n in 0..64 {
        let p = format!("/docs/n{n}");
        many.nodes.insert(p.clone(), node(false, 0o644, "alex"));
        many.files.insert(p, String::new());
    }
    assert_eq!(
        file_index::rebuild(&c, Some(&c), &t, &many)
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    let mut bad = c.clone();
    bad.root = "/dócuments".into();
    assert_eq!(bad.shape().unwrap_err().kind, ErrorKind::Unsupported);
}
#[test]
fn existing_only_inaccessible_parent_is_not_silently_reported_as_supported() {
    let (c, t) = fixture();
    let i = build(&c, &t);
    let mut current = t.clone();
    current.nodes.get_mut("/docs/current").unwrap().mode = 0;
    assert_eq!(
        file_index::locate(&i, &c, Some(&c), &t, &current, "release.txt", false)
            .unwrap()
            .status,
        Status::Found
    );
    assert_eq!(
        file_index::locate(&i, &c, Some(&c), &t, &current, "release.txt", true)
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
}

#[test]
fn literal_work_budget_refuses_large_products_without_saved_mutation() {
    let (c, t) = fixture();
    let mut v = serde_json::to_value(build(&c, &t)).unwrap();
    let long = "x".repeat(200);
    let mut names = Vec::new();
    for n in 0..60 {
        names.push(format!("/docs/{long}{n:02}"));
    }
    names.sort();
    v["entries"] = serde_json::to_value(names).unwrap();
    let i = file_index::Index::decode(&serde_json::to_vec(&v).unwrap(), &c, &t).unwrap();
    let before = serde_json::to_vec(&i).unwrap();
    assert_eq!(
        file_index::locate(&i, &c, Some(&c), &t, &t, &"x".repeat(128), false)
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(serde_json::to_vec(&i).unwrap(), before);
}
#[test]
fn missing_output_rights_and_source_file_collision_are_not_silent_success() {
    let (c, t) = fixture();
    let mut source = t.clone();
    source.nodes.get_mut("/work").unwrap().mode = 0o555;
    assert_eq!(
        c.validate_source(&source).unwrap_err().kind,
        ErrorKind::Permission
    );
    let mut source = t.clone();
    source
        .nodes
        .insert(c.database.clone(), node(false, 0o600, "alex"));
    source
        .files
        .insert(c.database.clone(), "existing important file".into());
    assert_eq!(
        c.validate_source(&source).unwrap_err().kind,
        ErrorKind::Invalid
    );
}

#[test]
fn privileged_source_is_an_explicit_boundary_not_a_claim_of_visibility_emulation() {
    let (c, mut t) = fixture();
    t.user = "root".into();
    assert!(
        !t.valid(),
        "The existing filesystem already excludes root callers"
    );
    assert_eq!(
        c.validate_source(&t).unwrap_err().kind,
        ErrorKind::Unsupported
    );
}
#[test]
fn saved_pruning_requires_the_same_canonical_effective_form_as_rebuild() {
    let (c, t) = fixture();
    let mut v = serde_json::to_value(build(&c, &t)).unwrap();
    v["pruned"] = serde_json::json!(["/docs/archive", "/docs/archive/sub", "/docs/private"]);
    assert!(file_index::Index::decode(&serde_json::to_vec(&v).unwrap(), &c, &t).is_err());
    let normalized = file_index::parse_config(
        "PRUNEPATHS=\"/docs/archive /docs/archive/sub /docs/private\"",
        &c.root,
    )
    .unwrap();
    assert_eq!(normalized.directories(), build(&c, &t).pruned());
}

#[test]
fn results_emit_siblings_before_descendants_and_visit_nested_directories_before_prefix_siblings() {
    let (c, mut t) = fixture();
    for path in ["/docs/current/nested", "/docs/current-other"] {
        t.nodes.insert(path.into(), node(true, 0o755, "alex"));
    }
    for path in [
        "/docs/current/nested/deep.txt",
        "/docs/current-other/sibling.txt",
    ] {
        t.nodes.insert(path.into(), node(false, 0o600, "alex"));
        t.files.insert(path.into(), "ordinary inert note\n".into());
    }
    assert!(t.valid());
    let index = build(&c, &t);
    let expected = [
        "/docs/archive",
        "/docs/archives",
        "/docs/current",
        "/docs/current-other",
        "/docs/locked",
        "/docs/private",
        "/docs/archives/retained.txt",
        "/docs/current/nested",
        "/docs/current/release.txt",
        "/docs/current/nested/deep.txt",
        "/docs/current-other/sibling.txt",
        "/docs/locked/item.txt",
    ];
    assert_eq!(index.entries(), expected);
    assert_eq!(
        file_index::locate(&index, &c, Some(&c), &t, &t, "docs", false)
            .unwrap()
            .paths,
        expected
    );
    assert!(file_index::matches_current(&index, &c, Some(&c), &t, &t).unwrap());
}
#[test]
fn imported_root_or_disconnected_or_flat_sorted_names_are_not_canonical_snapshot_state() {
    let (c, t) = fixture();
    let index = build(&c, &t);
    let original = serde_json::to_value(&index).unwrap();
    for names in [
        vec!["/docs".to_owned()],
        vec!["/docs/missing-parent/child.txt".to_owned()],
        {
            let mut names = index.entries().to_vec();
            names.sort();
            names
        },
    ] {
        let mut v = original.clone();
        v["entries"] = serde_json::to_value(names).unwrap();
        assert!(file_index::Index::decode(&serde_json::to_vec(&v).unwrap(), &c, &t).is_err());
    }
}
