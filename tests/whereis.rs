//! Fixed in-process bounded whereis tests; no host utility execution.
use dlms_learning_lab::{
    engine::Node,
    filesystem::{Access, Tree},
    whereis::{self, Category, Context, Directory, ErrorKind, Scan, Status},
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
        "/scan",
        "/scan/bin",
        "/scan/man",
        "/scan/source",
        "/scan/nested",
    ] {
        nodes.insert(p.into(), node(true, 0o755, "root", "root"));
    }
    for p in ["/home/alex", "/work"] {
        nodes.insert(p.into(), node(true, 0o700, "alex", "staff"));
    }
    let mut files = BTreeMap::new();
    for (p, body) in [
        ("/scan/bin/atlasctl", "inert binary metadata\n"),
        ("/scan/man/atlasctl.1", "current manual\n"),
        ("/scan/source/atlasctl.c", "current source\n"),
        ("/work/note", "private preserved note\n"),
    ] {
        nodes.insert(p.into(), node(false, 0o000, "alex", "staff"));
        files.insert(p.into(), body.into());
    }
    let tree = Tree {
        nodes,
        files,
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into()]),
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
    let context = Context {
        version: 1,
        source_id: "whereis-fixture".into(),
        names: vec!["atlasctl".into(), "missing".into()],
        directories: vec![
            Directory {
                category: Category::Binary,
                path: "/scan/bin".into(),
            },
            Directory {
                category: Category::Manual,
                path: "/scan/man".into(),
            },
            Directory {
                category: Category::Source,
                path: "/scan/source".into(),
            },
        ],
    };
    assert!(tree.valid());
    context.validate_for(&tree).unwrap();
    (context, tree)
}
fn run(c: &Context, source: &Tree, current: &Tree, input: &str) -> whereis::Output {
    whereis::execute_bound(c, Some(c), source, current, input).unwrap()
}
#[test]
fn three_categories_report_actual_names_without_running_or_reading_a_file() {
    let (c, t) = fixture();
    let before = t.clone();
    for (input, path) in [
        ("whereis -b -B /scan/bin -f atlasctl", "/scan/bin/atlasctl"),
        (
            "whereis -m -M /scan/man -f atlasctl",
            "/scan/man/atlasctl.1",
        ),
        (
            "whereis -s -S /scan/source -f atlasctl",
            "/scan/source/atlasctl.c",
        ),
    ] {
        let o = run(&c, &t, &t, input);
        assert_eq!(o.status, Status::Found);
        assert_eq!(o.exit_status, 0);
        assert!(o.command_succeeded());
        assert_eq!(o.paths, [path]);
        assert_eq!(o.stdout, format!("atlasctl: {path}\n"));
        assert!(o.matches(&c, &t, &t));
        assert!(o.diagnostic.contains("does not prove"));
    }
    assert_eq!(t, before);
}
#[test]
fn supported_missing_is_successful_query_with_empty_match_set_not_an_error() {
    let (c, t) = fixture();
    let o = run(&c, &t, &t, "whereis -m -M /scan/man -f missing");
    assert_eq!(o.status, Status::Missing);
    assert_eq!(o.stdout, "missing:\n");
    assert!(o.paths.is_empty());
    assert_eq!(o.exit_status, 0);
    assert!(o.command_succeeded());
    assert_eq!(o.scan, Scan::Listed);
    assert!(o.matches(&c, &t, &t));
}
#[test]
fn matching_directory_is_not_filtered_as_if_whereis_were_which() {
    let (c, t) = fixture();
    let mut current = t.clone();
    current.files.remove("/scan/bin/atlasctl");
    current
        .nodes
        .get_mut("/scan/bin/atlasctl")
        .unwrap()
        .directory = true;
    assert!(current.valid());
    assert_eq!(
        run(&c, &t, &current, "whereis -b -B /scan/bin -f atlasctl").paths,
        ["/scan/bin/atlasctl"]
    );
    assert!(!current.access("alex", "/scan/bin/atlasctl", Access::ExecuteFile));
}
#[test]
fn file_contents_execute_read_bits_and_owner_do_not_choose_the_matching_name() {
    let (c, t) = fixture();
    let mut current = t.clone();
    let n = current.nodes.get_mut("/scan/bin/atlasctl").unwrap();
    n.owner = "root".into();
    n.group = "root".into();
    n.mode = 0;
    current.files.insert(
        "/scan/bin/atlasctl".into(),
        "unrelated version bytes\n".into(),
    );
    assert!(!current.access("alex", "/scan/bin/atlasctl", Access::ReadFile));
    assert!(!current.access("alex", "/scan/bin/atlasctl", Access::ExecuteFile));
    assert_eq!(
        run(&c, &t, &current, "whereis -b -B /scan/bin -f atlasctl").paths,
        ["/scan/bin/atlasctl"]
    );
}
#[test]
fn directory_read_needs_ancestor_search_but_not_scanned_directory_execute() {
    let (c, t) = fixture();
    let mut current = t.clone();
    current.nodes.get_mut("/scan/bin").unwrap().mode = 0o444;
    assert!(current.access("alex", "/scan/bin", Access::ListNames));
    assert!(!current.access("alex", "/scan/bin", Access::SearchDirectory));
    assert_eq!(
        run(&c, &t, &current, "whereis -b -B /scan/bin -f atlasctl").status,
        Status::Found
    );
    current.nodes.get_mut("/scan").unwrap().mode = 0o700;
    let o = run(&c, &t, &current, "whereis -b -B /scan/bin -f atlasctl");
    assert_eq!(o.scan, Scan::InaccessibleDirectory);
    assert_eq!(o.stdout, "atlasctl:\n");
    assert_eq!(o.exit_status, 0);
}
#[test]
fn unreadable_directory_and_owner_class_precedence_skip_names() {
    let (c, t) = fixture();
    let mut current = t.clone();
    current.nodes.get_mut("/scan/bin").unwrap().mode = 0o111;
    assert_eq!(
        run(&c, &t, &current, "whereis -b -B /scan/bin -f atlasctl").scan,
        Scan::InaccessibleDirectory
    );
    let n = current.nodes.get_mut("/scan/bin").unwrap();
    n.owner = "alex".into();
    n.group = "staff".into();
    n.mode = 0o055;
    assert!(!current.access("alex", "/scan/bin", Access::ListNames));
    assert_eq!(
        run(&c, &t, &current, "whereis -b -B /scan/bin -f atlasctl").status,
        Status::Missing
    );
}
#[test]
fn missing_directory_and_regular_file_directory_argument_are_distinct_empty_scans() {
    let (mut c, t) = fixture();
    c.directories[0].path = "/absent".into();
    let o = run(&c, &t, &t, "whereis -b -B /absent -f atlasctl");
    assert_eq!(o.scan, Scan::MissingDirectory);
    assert_eq!(o.exit_status, 0);
    c.directories[0].path = "/work/note".into();
    let o = run(&c, &t, &t, "whereis -b -B /work/note -f atlasctl");
    assert_eq!(o.scan, Scan::NotDirectory);
    assert_eq!(o.stdout, "atlasctl:\n");
}
#[test]
fn lookup_is_immediate_only_case_sensitive_and_binary_suffix_does_not_match() {
    let (c, t) = fixture();
    let mut current = t.clone();
    current.files.remove("/scan/bin/atlasctl");
    current.nodes.remove("/scan/bin/atlasctl");
    for p in [
        "/scan/bin/atlasctl.old",
        "/scan/bin/Atlasctl",
        "/scan/nested/atlasctl",
    ] {
        current
            .nodes
            .insert(p.into(), node(false, 0o755, "alex", "staff"));
        current.files.insert(p.into(), "not a match\n".into());
    }
    assert_eq!(
        run(&c, &t, &current, "whereis -b -B /scan/bin -f atlasctl").status,
        Status::Missing
    );
}
#[test]
fn manual_sections_compression_and_exact_unsuffixed_name_are_honestly_unsupported() {
    let (c, t) = fixture();
    for p in [
        "/scan/man/atlasctl",
        "/scan/man/atlasctl.8",
        "/scan/man/atlasctl.1.gz",
        "/scan/man/atlasctl.1.xz",
        "/scan/man/atlasctl.extra.C",
        "/scan/man/atlasctl..unproved",
    ] {
        let mut current = t.clone();
        current
            .nodes
            .insert(p.into(), node(false, 0o644, "alex", "staff"));
        current.files.insert(p.into(), "x".into());
        let before = current.clone();
        assert_eq!(
            whereis::execute_bound(
                &c,
                Some(&c),
                &t,
                &current,
                "whereis -m -M /scan/man -f atlasctl"
            )
            .unwrap_err()
            .kind,
            ErrorKind::Unsupported
        );
        assert_eq!(current, before);
    }
}
#[test]
fn source_prefix_other_extensions_and_multiple_possible_matches_do_not_return_partial_results() {
    let (c, t) = fixture();
    for p in [
        "/scan/source/s.atlasctl.c",
        "/scan/source/s.s.atlasctl.c",
        "/scan/source/atlasctl",
        "/scan/source/atlasctl.h",
        "/scan/source/atlasctl.c.gz",
    ] {
        let mut current = t.clone();
        current
            .nodes
            .insert(p.into(), node(false, 0o644, "alex", "staff"));
        current.files.insert(p.into(), "x".into());
        assert_eq!(
            whereis::execute_bound(
                &c,
                Some(&c),
                &t,
                &current,
                "whereis -s -S /scan/source -f atlasctl"
            )
            .unwrap_err()
            .kind,
            ErrorKind::Unsupported
        );
    }
}
#[test]
fn parser_accepts_only_exact_supplied_category_directory_and_literal_basename() {
    let (c, t) = fixture();
    for input in [
        "whereis atlasctl",
        "whereis -b atlasctl",
        "whereis -b -B /scan/bin atlasctl",
        "whereis -b -B /scan/bin /scan/source -f atlasctl",
        "whereis -b -m -B /scan/bin -f atlasctl",
        "whereis -m -M /scan/bin -f atlasctl",
        "whereis -b -B /scan/bin -f unknown",
        "whereis -b -B /scan/bin -f /scan/bin/atlasctl",
        "whereis -b -B /scan/bin -f atlasctl missing",
        "whereis -u -b -B /scan/bin -f atlasctl",
        "whereis -g -b -B /scan/bin -f atlasctl",
        "whereis -b -B /scan/bin -f atlasctl; id",
        "whereis -b -B /scan/bin -f $(id)",
        "whereis -b -B /scan/bin -f atlasctl | cat",
        "whereis -b -B /scan/bin -f atlasctl > /work/note",
    ] {
        assert_eq!(
            whereis::execute_bound(&c, Some(&c), &t, &t, input)
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported,
            "{input}"
        );
    }
    assert_eq!(
        run(&c, &t, &t, "whereis -b -B '/scan/bin' -f 'atlasctl'").status,
        Status::Found
    );
}
#[test]
fn category_only_context_has_no_implicit_other_category_dependency() {
    let (mut c, t) = fixture();
    c.directories.retain(|d| d.category == Category::Source);
    assert_eq!(
        run(&c, &t, &t, "whereis -s -S /scan/source -f atlasctl").status,
        Status::Found
    );
    assert_eq!(
        whereis::execute_bound(&c, Some(&c), &t, &t, "whereis -b -B /scan/bin -f atlasctl")
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
}
#[test]
fn imported_metadata_cannot_replace_or_erase_the_held_context() {
    let (c, t) = fixture();
    let mut forged = c.clone();
    forged.source_id = "other-example".into();
    for carried in [None, Some(&forged)] {
        assert_eq!(
            whereis::execute_bound(&c, carried, &t, &t, "whereis -b -B /scan/bin -f atlasctl")
                .unwrap_err()
                .kind,
            ErrorKind::SourceMismatch
        );
    }
    let mut current = t.clone();
    current.user = "blair".into();
    assert_eq!(
        whereis::execute_bound(
            &c,
            Some(&c),
            &t,
            &current,
            "whereis -b -B /scan/bin -f atlasctl"
        )
        .unwrap_err()
        .kind,
        ErrorKind::SourceMismatch
    );
}
#[test]
fn current_operational_marker_cannot_be_added_swapped_or_erased() {
    let (c, t) = fixture();
    let mut source = t.clone();
    source.command_location = Some(dlms_learning_lab::command_location::Context {
        version: 1,
        source_id: "separate-mode-source".into(),
        path: "/scan/bin".into(),
        names: vec!["atlasctl".into()],
    });
    assert!(source.valid());
    assert_eq!(
        whereis::execute_bound(
            &c,
            Some(&c),
            &source,
            &t,
            "whereis -b -B /scan/bin -f atlasctl"
        )
        .unwrap_err()
        .kind,
        ErrorKind::SourceMismatch
    );
    assert_eq!(
        whereis::execute_bound(
            &c,
            Some(&c),
            &t,
            &source,
            "whereis -b -B /scan/bin -f atlasctl"
        )
        .unwrap_err()
        .kind,
        ErrorKind::SourceMismatch
    );
}
#[test]
fn output_recomputation_invalidates_stale_or_forged_lookup_evidence() {
    let (c, t) = fixture();
    let o = run(&c, &t, &t, "whereis -m -M /scan/man -f atlasctl");
    let mut current = t.clone();
    current.files.remove("/scan/man/atlasctl.1");
    current.nodes.remove("/scan/man/atlasctl.1");
    assert!(!o.matches(&c, &t, &current));
    let mut fake = o.clone();
    fake.exit_status = 1;
    assert!(!fake.matches(&c, &t, &t));
    let mut fake = o.clone();
    fake.paths.clear();
    assert!(!fake.matches(&c, &t, &t));
}
#[test]
fn malformed_contexts_names_paths_and_duplicate_categories_are_refused_before_search() {
    let (c, t) = fixture();
    for n in [
        "",
        "atlas.ctl",
        "s.atlas",
        "../atlas",
        "$atlas",
        "álas",
        "atlas/ctl",
    ] {
        let mut bad = c.clone();
        bad.names = vec![n.into()];
        assert!(bad.validate_for(&t).is_err(), "{n}");
    }
    for dir in [
        "",
        "relative",
        "/scan/../bin",
        "/scan//bin",
        "/scan/bin/",
        "/scan/\0bin",
    ] {
        let mut bad = c.clone();
        bad.directories[0].path = dir.into();
        assert!(bad.validate_for(&t).is_err(), "{dir}");
    }
    let mut bad = c.clone();
    bad.names.push("atlasctl".into());
    assert!(bad.shape().is_err());
    let mut bad = c.clone();
    bad.directories.push(bad.directories[0].clone());
    assert!(bad.shape().is_err());
    let mut bad = c.clone();
    bad.version = 2;
    assert!(bad.shape().is_err());
}
#[test]
fn bounded_decode_rejects_raw_size_typed_overflow_unknown_and_duplicate_fields() {
    let (c, t) = fixture();
    let raw = serde_json::to_vec(&c).unwrap();
    assert_eq!(Context::decode(&raw, &t).unwrap(), c);
    assert_eq!(
        Context::decode(&vec![b' '; whereis::MAX_CONTEXT_BYTES + 1], &t)
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    let mut v = serde_json::to_value(&c).unwrap();
    v["names"] = serde_json::json!(["a", "b", "c", "d", "e"]);
    assert!(Context::decode(&serde_json::to_vec(&v).unwrap(), &t).is_err());
    let mut v = serde_json::to_value(&c).unwrap();
    v["extra"] = true.into();
    assert!(Context::decode(&serde_json::to_vec(&v).unwrap(), &t).is_err());
    let duplicate = format!(
        "{{\"version\":1,{}",
        String::from_utf8(raw).unwrap().trim_start_matches('{')
    );
    assert!(Context::decode(duplicate.as_bytes(), &t).is_err());
}
#[test]
fn resource_limits_fail_without_mutation_or_host_fallback() {
    let (c, t) = fixture();
    assert_eq!(
        whereis::execute_bound(
            &c,
            Some(&c),
            &t,
            &t,
            &"x".repeat(whereis::MAX_INPUT_BYTES + 1)
        )
        .unwrap_err()
        .kind,
        ErrorKind::Limit
    );
    let mut big = t.clone();
    for n in 0..65 {
        let p = format!("/work/file{n}");
        big.nodes
            .insert(p.clone(), node(false, 0o600, "alex", "staff"));
        big.files.insert(p, "x".into());
    }
    assert_eq!(c.validate_for(&big).unwrap_err().kind, ErrorKind::Limit);
    let mut big = t.clone();
    big.files
        .insert("/work/note".into(), "x".repeat(256 * 1024 + 1));
    assert_eq!(c.validate_for(&big).unwrap_err().kind, ErrorKind::Limit);
    let before = t.clone();
    assert_eq!(t, before);
}
#[test]
fn canonical_unicode_directory_and_root_directory_are_literal_supported_paths() {
    let (mut c, t) = fixture();
    let mut current = t.clone();
    current
        .nodes
        .insert("/équipe".into(), node(true, 0o755, "root", "root"));
    current.nodes.insert(
        "/équipe/atlasctl".into(),
        node(false, 0o000, "alex", "staff"),
    );
    current
        .files
        .insert("/équipe/atlasctl".into(), "data".into());
    c.directories[0].path = "/équipe".into();
    assert_eq!(
        run(&c, &t, &current, "whereis -b -B /équipe -f atlasctl").paths,
        ["/équipe/atlasctl"]
    );
    current
        .nodes
        .insert("/atlasctl".into(), node(false, 0o000, "alex", "staff"));
    current.files.insert("/atlasctl".into(), "data".into());
    c.directories[0].path = "/".into();
    assert_eq!(
        run(&c, &t, &current, "whereis -b -B / -f atlasctl").paths,
        ["/atlasctl"]
    );
}
