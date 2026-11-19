//! Private pure-core proposal tests; no host utilities or command execution engine.
use dlms_learning_lab::{
    command_location::{self, Context, ErrorKind, Status},
    engine::Node,
    filesystem::{
        Access, Tree,
        path_lookup::{self, Decision, Profile},
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
fn run(c: &Context, source: &Tree, current: &Tree, input: &str) -> command_location::Output {
    command_location::execute_bound(c, Some(c), source, current, input).unwrap()
}
#[test]
fn which_uses_current_execute_permissions_and_keeps_all_file_bytes() {
    let (c, source) = fixture();
    let mut current = source.clone();
    let before = current.clone();
    let first = run(&c, &source, &current, "which report");
    assert_eq!(first.paths, ["/usr/bin/report"]);
    assert_eq!(first.stdout, "/usr/bin/report\n");
    assert_eq!(first.status, Status::Found);
    assert!(first.diagnostic.is_empty());
    assert_eq!(current, before);
    current.execute("chmod u+x /usr/local/bin/report").unwrap();
    let next = run(&c, &source, &current, "which report");
    assert_eq!(next.paths, ["/usr/local/bin/report"]);
    assert!(!first.matches(&c, &source, &current));
    assert!(next.matches(&c, &source, &current));
    assert_eq!(current.files, source.files);
    assert_eq!(
        run(&c, &source, &current, "which audit").paths,
        ["/usr/bin/audit"]
    );
}
#[test]
fn all_paths_are_real_eligible_matches_in_path_order_not_first_trace() {
    let (c, source) = fixture();
    let mut current = source.clone();
    assert_eq!(
        run(&c, &source, &current, "type -a -p report").paths,
        ["/usr/bin/report"]
    );
    current.execute("chmod 0744 /usr/local/bin/report").unwrap();
    let o = run(&c, &source, &current, "type -a -p report");
    assert_eq!(o.paths, ["/usr/local/bin/report", "/usr/bin/report"]);
    assert_eq!(o.stdout, "/usr/local/bin/report\n/usr/bin/report\n");
    assert_eq!(o.candidates.len(), 2);
    assert!(o.steps > 0 && o.steps <= path_lookup::MAX_STEPS);
    let mut reversed = c.clone();
    reversed.path = "/usr/bin:/usr/local/bin".into();
    assert_eq!(
        run(&reversed, &source, &current, "type -a -p report").paths,
        ["/usr/bin/report", "/usr/local/bin/report"]
    );
    assert_eq!(
        run(&c, &source, &current, "which report").candidates.len(),
        1
    );
}
#[test]
fn supported_missing_is_not_unsupported_or_false_success() {
    let (c, source) = fixture();
    for input in ["which missing", "type -a -p missing"] {
        let o = run(&c, &source, &source, input);
        assert_eq!(o.status, Status::Missing);
        assert!(!o.success());
        assert!(o.paths.is_empty());
        assert!(o.stdout.is_empty());
        assert!(o.diagnostic.starts_with("Simulator lookup:"));
        assert!(o.matches(&c, &source, &source));
    }
    let error = command_location::execute_bound(&c, Some(&c), &source, &source, "which unknown")
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Unsupported);
}
#[test]
fn unsupported_commands_options_composition_and_explicit_paths_never_run() {
    let (c, source) = fixture();
    let before = source.clone();
    for text in [
        "type report",
        "type -P report",
        "type -p report",
        "type -ap report",
        "which -a report",
        "which -- report",
        "whereis report",
        "which report audit",
        "which /usr/bin/report",
        "which report; rm /work/note",
        "which $(id)",
        "which report | cat",
        "which report > /work/result",
        "env PATH=/usr/bin which report",
        "",
    ] {
        assert_eq!(
            command_location::execute_bound(&c, Some(&c), &source, &source, text)
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported,
            "{text}"
        );
    }
    assert_eq!(source, before);
    assert_eq!(
        run(&c, &source, &source, "which 'report'").paths,
        ["/usr/bin/report"]
    );
}
#[test]
fn empty_duplicate_relative_and_noncanonical_path_refuse_upfront_even_after_usable_prefix() {
    let (c, source) = fixture();
    for raw in [
        "",
        ":/usr/bin",
        "/usr/bin:",
        "relative:/usr/bin",
        "/usr/bin//",
        "/usr/../usr/bin",
        "/usr/bin:/usr/bin",
        "/usr/bin:/missing/../bin",
    ] {
        let mut bad = c.clone();
        bad.path = raw.into();
        assert_eq!(
            bad.validate_for(&source).unwrap_err().kind,
            ErrorKind::Unsupported,
            "{raw}"
        );
        assert!(
            command_location::execute_bound(&bad, Some(&bad), &source, &source, "which report")
                .is_err()
        );
    }
}
#[test]
fn ancestor_search_and_file_execute_do_not_need_read() {
    let (mut c, source) = fixture();
    c.path = "/search:/usr/bin".into();
    assert!(!source.access("alex", "/search", Access::ListNames));
    assert!(!source.access("alex", "/search/report", Access::ReadFile));
    assert!(source.access("alex", "/search/report", Access::ExecuteFile));
    assert_eq!(
        run(&c, &source, &source, "which report").paths,
        ["/search/report"]
    );
    c.path = "/denied:/targetdir:/usr/bin".into();
    let o = run(&c, &source, &source, "type -a -p report");
    assert_eq!(o.paths, ["/usr/bin/report"]);
    assert_eq!(o.candidates[0].state, path_lookup::CandidateState::Denied);
    assert_eq!(
        o.candidates[1].state,
        path_lookup::CandidateState::NotRegularFile
    );
    c.path = "/usr/bin/report:/usr/bin".into();
    let o = run(&c, &source, &source, "which report");
    assert_eq!(o.paths, ["/usr/bin/report"]);
    assert_eq!(
        o.candidates[0].state,
        path_lookup::CandidateState::NotDirectory
    );
}
#[test]
fn owner_group_other_execute_precedence_reuses_existing_access() {
    let (c, source) = fixture();
    for (owner, group, mode, wanted) in [
        ("alex", "ops", 0o010, false),
        ("blair", "ops", 0o010, true),
        ("blair", "root", 0o001, true),
        ("alex", "staff", 0o100, true),
        ("blair", "staff", 0o001, false),
    ] {
        let mut current = source.clone();
        let node = current.nodes.get_mut("/usr/local/bin/report").unwrap();
        node.owner = owner.into();
        node.group = group.into();
        node.mode = mode;
        assert_eq!(
            current.access("alex", "/usr/local/bin/report", Access::ExecuteFile),
            wanted
        );
        let paths = run(&c, &source, &current, "type -a -p report").paths;
        assert_eq!(paths.contains(&"/usr/local/bin/report".into()), wanted);
    }
}
#[test]
fn source_metadata_erasure_forgery_and_changed_caller_foundation_refuse() {
    let (c, source) = fixture();
    assert_eq!(
        command_location::execute_bound(&c, None, &source, &source, "which report")
            .unwrap_err()
            .kind,
        ErrorKind::SourceMismatch
    );
    let mut foreign = c.clone();
    foreign.source_id = "foreign".into();
    assert_eq!(
        command_location::execute_bound(&c, Some(&foreign), &source, &source, "which report")
            .unwrap_err()
            .kind,
        ErrorKind::SourceMismatch
    );
    for kind in 0..3 {
        let mut current = source.clone();
        match kind {
            0 => current.user = "blair".into(),
            1 => {
                current.users.get_mut("alex").unwrap().pop();
            }
            _ => current.home = "/work".into(),
        }
        assert!(current.valid());
        assert_eq!(
            command_location::execute_bound(&c, Some(&c), &source, &current, "which report")
                .unwrap_err()
                .kind,
            ErrorKind::SourceMismatch
        );
    }
    let mut current = source.clone();
    current.cwd = "/search".into();
    assert!(run(&c, &source, &current, "which report").success());
}
#[test]
fn output_is_not_a_stored_pass_flag_and_foreign_or_changed_results_fail_recompute() {
    let (c, source) = fixture();
    let o = run(&c, &source, &source, "which missing");
    let mut forged = o.clone();
    forged.status = Status::Found;
    forged.paths = vec!["/usr/bin/report".into()];
    forged.stdout = "/usr/bin/report\n".into();
    assert!(!forged.matches(&c, &source, &source));
    let mut foreign = c.clone();
    foreign.source_id = "elsewhere".into();
    assert!(!o.matches(&foreign, &source, &source));
    let mut forged = o.clone();
    forged.steps = 0;
    assert!(!forged.matches(&c, &source, &source));
}
#[test]
fn large_valid_file_contents_are_not_discarded_or_returned_by_metadata_lookup() {
    let (c, source) = fixture();
    let mut current = source.clone();
    for content in current.files.values_mut() {
        content.clear();
    }
    current
        .files
        .insert("/usr/local/bin/report".into(), "p".repeat(131072));
    current
        .files
        .insert("/usr/bin/report".into(), "f".repeat(131072));
    assert!(current.valid());
    let before = current.files.clone();
    let result = run(&c, &source, &current, "which report");
    assert_eq!(result.stdout, "/usr/bin/report\n");
    assert_eq!(current.files, before);
}
#[test]
fn context_raw_bounds_duplicates_reserved_names_and_unknown_fields_refuse() {
    let (c, source) = fixture();
    let raw = serde_json::to_vec(&c).unwrap();
    assert_eq!(Context::decode(&raw, &source).unwrap(), c);
    let mut bad = c.clone();
    bad.names = vec![];
    assert!(bad.shape().is_err());
    bad.names = vec!["report".into(); 5];
    assert!(bad.shape().is_err());
    bad.names = vec!["report".into(), "report".into()];
    assert!(bad.shape().is_err());
    bad.names = vec!["réport".into()];
    assert!(bad.shape().is_err());
    bad.names = vec!["type".into()];
    assert!(bad.validate_for(&source).is_err());
    let mut v = serde_json::to_value(&c).unwrap();
    v["extra"] = true.into();
    assert!(Context::decode(&serde_json::to_vec(&v).unwrap(), &source).is_err());
    let raw = br#"{"version":1,"version":1,"source_id":"x","path":"/usr/bin","names":["report"]}"#;
    assert!(Context::decode(raw, &source).is_err());
    assert_eq!(
        Context::decode(
            &vec![b' '; command_location::MAX_CONTEXT_BYTES + 1],
            &source
        )
        .unwrap_err()
        .kind,
        ErrorKind::Limit
    );
}
#[test]
fn input_path_candidate_and_live_tree_limits_are_explicit() {
    let (c, source) = fixture();
    assert_eq!(
        command_location::execute_bound(
            &c,
            Some(&c),
            &source,
            &source,
            &"x".repeat(command_location::MAX_INPUT_BYTES + 1)
        )
        .unwrap_err()
        .kind,
        ErrorKind::Limit
    );
    let mut bad = c.clone();
    bad.path = "x".repeat(16385);
    assert_eq!(bad.shape().unwrap_err().kind, ErrorKind::Limit);
    let mut bad = c.clone();
    bad.path = (0..17)
        .map(|i| format!("/d{i}"))
        .collect::<Vec<_>>()
        .join(":");
    assert_eq!(bad.shape().unwrap_err().kind, ErrorKind::Limit);
    let mut current = source.clone();
    for i in 0..70 {
        current
            .nodes
            .insert(format!("/extra{i}"), node(true, 0o555, "root", "root"));
    }
    assert_eq!(
        command_location::execute_bound(&c, Some(&c), &source, &current, "which report")
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    let mut root = source.clone();
    root.user = "root".into();
    assert_eq!(
        command_location::execute_bound(&c, Some(&c), &source, &root, "which report")
            .unwrap_err()
            .kind,
        ErrorKind::Invalid
    );
}
#[test]
fn sixteen_all_result_directories_are_bounded_and_ordered() {
    let (mut c, mut source) = fixture();
    c.names = vec!["report".into()];
    let mut expected = vec![];
    for i in 0..16 {
        let dir = format!("/d{i}");
        let file = format!("{dir}/report");
        source.nodes.insert(dir, node(true, 0o555, "root", "root"));
        source
            .nodes
            .insert(file.clone(), node(false, 0o111, "alex", "staff"));
        source.files.insert(file.clone(), String::new());
        expected.push(file);
    }
    assert!(source.nodes.len() <= 64);
    c.path = (0..16)
        .map(|i| format!("/d{i}"))
        .collect::<Vec<_>>()
        .join(":");
    let result = run(&c, &source, &source, "type -a -p report");
    assert_eq!(result.paths, expected);
    assert_eq!(result.candidates.len(), 16);
    assert!(result.stdout.len() <= command_location::MAX_OUTPUT_BYTES);
    assert!(result.steps <= path_lookup::MAX_STEPS);
}
#[test]
fn unicode_directory_data_is_supported_but_query_names_remain_ascii() {
    let (mut c, mut source) = fixture();
    source
        .nodes
        .insert("/équipe".into(), node(true, 0o555, "root", "root"));
    source
        .nodes
        .insert("/équipe/report".into(), node(false, 0o111, "alex", "staff"));
    source
        .files
        .insert("/équipe/report".into(), "virtual artifact".into());
    c.path = "/équipe:/usr/bin".into();
    assert_eq!(
        run(&c, &source, &source, "which report").stdout,
        "/équipe/report\n"
    );
}
#[test]
fn old_resolver_still_stops_at_first_usable_provider() {
    let (c, source) = fixture();
    let mut metadata = source.clone();
    for text in metadata.files.values_mut() {
        text.clear();
    }
    let p = Profile {
        version: 1,
        tree: metadata,
        trusted_directories: vec![],
        names: vec!["report".into()],
    };
    let observation = path_lookup::resolve(&p, Some(&c.path)).unwrap();
    assert!(
        matches!(&observation.reports[0].decision,Decision::Selected{path}if path=="/usr/bin/report")
    );
    assert_eq!(observation.reports[0].candidates.len(), 2);
    assert_eq!(
        run(&c, &source, &source, "which report").candidates,
        observation.reports[0].candidates
    );
}

#[test]
fn projection_never_launders_invalid_file_content_or_current_permission_denial() {
    let (mut c, source) = fixture();
    let mut bad = source.clone();
    bad.files.get_mut("/usr/bin/report").unwrap().push('\0');
    assert_eq!(
        command_location::execute_bound(&c, Some(&c), &source, &bad, "which report")
            .unwrap_err()
            .kind,
        ErrorKind::Invalid
    );
    c.path = "/denied".into();
    let o = run(&c, &source, &source, "which report");
    assert_eq!(o.status, Status::Missing);
    assert!(!o.success());
    assert!(o.stdout.is_empty());
    assert_eq!(o.candidates[0].state, path_lookup::CandidateState::Denied);
    let mut carried = c.clone();
    carried.path = "/usr/bin".into();
    assert_eq!(
        command_location::execute_bound(&c, Some(&carried), &source, &source, "which report")
            .unwrap_err()
            .kind,
        ErrorKind::SourceMismatch
    );
}
