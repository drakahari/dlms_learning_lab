//! Pure virtual PATH metadata: no host lookup or utility execution.
use dlms_learning_lab::{
    engine::Node,
    filesystem::{
        Access, Tree,
        path_lookup::{self, CandidateState, Decision, ErrorKind, Observation, Policy, Profile},
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
fn profile() -> Profile {
    let mut nodes = BTreeMap::new();
    for p in [
        "/",
        "/home",
        "/home/alex",
        "/work",
        "/usr",
        "/usr/bin",
        "/opt",
        "/opt/new",
        "/opt/new/bin",
        "/opt/old",
        "/opt/old/bin",
        "/opt/extra",
        "/opt/extra/bin",
        "/denied",
        "/search",
        "/targetdir",
        "/targetdir/report",
    ] {
        nodes.insert(p.into(), node(true, 0o555, "root", "root"));
    }
    nodes.insert("/home/alex".into(), node(true, 0o700, "alex", "staff"));
    nodes.insert("/work".into(), node(true, 0o755, "alex", "staff"));
    nodes.get_mut("/denied").unwrap().mode = 0o700;
    nodes.get_mut("/search").unwrap().mode = 0o111;
    let mut files = BTreeMap::new();
    for (p, mode) in [
        ("/usr/bin/audit", 0o555),
        ("/usr/bin/report", 0o644),
        ("/opt/new/bin/report", 0o111),
        ("/opt/old/bin/report", 0o555),
        ("/opt/extra/bin/report", 0o555),
        ("/denied/report", 0o555),
        ("/search/report", 0o111),
        ("/opt/notdir", 0o555),
    ] {
        nodes.insert(p.into(), node(false, mode, "root", "root"));
        files.insert(p.into(), String::new());
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
    let p = Profile {
        version: 1,
        tree,
        trusted_directories: [
            "/usr/bin",
            "/opt/new/bin",
            "/opt/extra/bin",
            "/denied",
            "/search",
            "/targetdir",
        ]
        .map(str::to_owned)
        .to_vec(),
        names: vec!["report".into(), "audit".into()],
    };
    p.validate().unwrap();
    p
}
fn selected(o: &Observation, index: usize) -> &str {
    let Decision::Selected { path } = &o.reports[index].decision else {
        panic!("expected selected path")
    };
    path
}
#[test]
fn exact_ordered_x_file_lookup_and_other_tool_preservation_are_read_only() {
    let p = profile();
    let before = p.clone();
    let o = path_lookup::resolve(&p, Some("/opt/new/bin:/usr/bin")).unwrap();
    assert_eq!(selected(&o, 0), "/opt/new/bin/report");
    assert_eq!(selected(&o, 1), "/usr/bin/audit");
    assert_eq!(o.policy, Policy::Trusted);
    assert_eq!(
        o.reports[1]
            .candidates
            .iter()
            .map(|c| c.state)
            .collect::<Vec<_>>(),
        vec![CandidateState::Missing, CandidateState::Selected]
    );
    assert!(o.steps > 0 && o.steps <= path_lookup::MAX_STEPS);
    assert_eq!(p, before);
}
#[test]
fn alternative_strings_duplicate_components_and_multiple_approved_copies_are_real_decisions() {
    let p = profile();
    for path in [
        "/opt/new/bin:/usr/bin",
        "/usr/bin:/opt/new/bin",
        "/opt/new/bin:/opt/new/bin:/usr/bin",
    ] {
        let o = path_lookup::resolve(&p, Some(path)).unwrap();
        assert_eq!(selected(&o, 0), "/opt/new/bin/report");
        assert_eq!(selected(&o, 1), "/usr/bin/audit");
        assert_eq!(o.policy, Policy::Trusted);
    }
    let duplicated = path_lookup::resolve(&p, Some("/opt/new/bin:/opt/new/bin:/usr/bin")).unwrap();
    assert_eq!(
        duplicated.reports[1].candidates.len(),
        3,
        "Do not silently deduplicate searched components"
    );
    for directory in ["/opt/new/bin", "/opt/extra/bin"] {
        let o = path_lookup::resolve(&p, Some(&format!("{directory}:/usr/bin"))).unwrap();
        assert!(["/opt/new/bin/report", "/opt/extra/bin/report"].contains(&selected(&o, 0)));
    }
}
#[test]
fn first_usable_provider_wins_but_policy_checks_even_an_unused_untrusted_suffix() {
    let p = profile();
    let old = path_lookup::resolve(&p, Some("/opt/old/bin:/opt/new/bin:/usr/bin")).unwrap();
    assert_eq!(selected(&old, 0), "/opt/old/bin/report");
    assert_eq!(old.reports[0].candidates.len(), 1);
    let suffix = path_lookup::resolve(&p, Some("/opt/new/bin:/usr/bin:/opt/old/bin")).unwrap();
    assert_eq!(selected(&suffix, 0), "/opt/new/bin/report");
    assert_eq!(selected(&suffix, 1), "/usr/bin/audit");
    assert_eq!(
        suffix.policy,
        Policy::Untrusted {
            directories: vec!["/opt/old/bin".into()]
        }
    );
    assert_eq!(suffix.components.len(), 3);
}
#[test]
fn file_execute_and_parent_search_do_not_require_read_permissions() {
    let mut p = profile();
    p.names = vec!["report".into()];
    assert!(
        !p.tree
            .access("alex", "/opt/new/bin/report", Access::ReadFile)
    );
    assert!(
        p.tree
            .access("alex", "/opt/new/bin/report", Access::ExecuteFile)
    );
    assert!(!p.tree.access("alex", "/search", Access::ListNames));
    assert!(p.tree.access("alex", "/search", Access::SearchDirectory));
    let o = path_lookup::resolve(&p, Some("/search")).unwrap();
    assert_eq!(selected(&o, 0), "/search/report");
    assert!(
        !p.tree.access("alex", "/search", Access::ExecuteFile),
        "directory is not an executable artifact"
    );
}
#[test]
fn owner_group_other_execute_class_precedence_matches_existing_tree_permissions() {
    let mut p = profile();
    p.names = vec!["report".into()];
    let file = "/opt/new/bin/report";
    for (owner, group, mode, pass) in [
        ("alex", "staff", 0o001, false),
        ("blair", "ops", 0o001, false),
        ("blair", "ops", 0o010, true),
        ("blair", "root", 0o001, true),
        ("alex", "staff", 0o100, true),
    ] {
        *p.tree.nodes.get_mut(file).unwrap() = node(false, mode, owner, group);
        p.validate().unwrap();
        assert_eq!(p.tree.access("alex", file, Access::ExecuteFile), pass);
        let o = path_lookup::resolve(&p, Some("/opt/new/bin")).unwrap();
        assert_eq!(
            matches!(o.reports[0].decision, Decision::Selected { .. }),
            pass
        );
    }
}
#[test]
fn denied_ancestor_and_nonexecutable_candidates_are_skipped_before_later_usable_files() {
    let mut p = profile();
    p.names = vec!["report".into()];
    let denied = path_lookup::resolve(&p, Some("/denied:/opt/new/bin")).unwrap();
    assert_eq!(
        denied.reports[0].candidates[0].state,
        CandidateState::Denied
    );
    assert_eq!(selected(&denied, 0), "/opt/new/bin/report");
    let no_x = path_lookup::resolve(&p, Some("/usr/bin:/opt/new/bin")).unwrap();
    assert_eq!(
        no_x.reports[0].candidates[0].state,
        CandidateState::NotExecutable
    );
    assert_eq!(selected(&no_x, 0), "/opt/new/bin/report");
    let denied = path_lookup::resolve(&p, Some("/denied")).unwrap();
    assert_eq!(denied.reports[0].decision, Decision::Denied);
}
#[test]
fn missing_nondirectory_and_directory_artifacts_remain_distinct_typed_trace_facts() {
    let mut p = profile();
    p.names = vec!["report".into()];
    let o = path_lookup::resolve(&p, Some("/missing:/opt/notdir:/targetdir:/usr/bin")).unwrap();
    assert_eq!(
        o.reports[0]
            .candidates
            .iter()
            .map(|c| c.state)
            .collect::<Vec<_>>(),
        vec![
            CandidateState::Missing,
            CandidateState::NotDirectory,
            CandidateState::NotRegularFile,
            CandidateState::NotExecutable
        ]
    );
    assert_eq!(o.reports[0].decision, Decision::Denied);
    let absent = path_lookup::resolve(&p, Some("/nowhere")).unwrap();
    assert_eq!(absent.reports[0].decision, Decision::NotFound);
}
#[test]
fn unsupported_unset_empty_relative_and_noncanonical_path_never_become_defaults() {
    let p = profile();
    for path in [
        None,
        Some(""),
        Some(":"),
        Some("/usr/bin:"),
        Some(":/usr/bin"),
        Some("/usr/bin::/opt/new/bin"),
        Some("bin"),
        Some("./bin"),
        Some("/usr/../bin"),
        Some("/usr//bin"),
        Some("/usr/bin/"),
        Some("/a\n/b"),
    ] {
        assert_eq!(
            path_lookup::resolve(&p, path).unwrap_err().kind,
            ErrorKind::Unsupported,
            "{path:?}"
        );
    }
    assert_eq!(
        path_lookup::resolve(&p, Some(&"/x".repeat(8193)))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    let too_many = std::iter::repeat_n("/usr/bin", 17)
        .collect::<Vec<_>>()
        .join(":");
    assert_eq!(
        path_lookup::resolve(&p, Some(&too_many)).unwrap_err().kind,
        ErrorKind::Limit
    );
}
#[test]
fn canonical_unicode_directories_work_but_fixed_query_names_are_ascii_external_only() {
    let mut p = profile();
    for dir in ["/opt/έκθεση", "/opt/έκθεση/bin"] {
        p.tree
            .nodes
            .insert(dir.into(), node(true, 0o555, "root", "root"));
    }
    p.tree.nodes.insert(
        "/opt/έκθεση/bin/report".into(),
        node(false, 0o111, "root", "root"),
    );
    p.tree
        .files
        .insert("/opt/έκθεση/bin/report".into(), String::new());
    p.trusted_directories.push("/opt/έκθεση/bin".into());
    p.validate().unwrap();
    let o = path_lookup::resolve(&p, Some("/opt/έκθεση/bin:/usr/bin")).unwrap();
    assert_eq!(selected(&o, 0), "/opt/έκθεση/bin/report");
    for name in [
        "résumé",
        "/bin/report",
        "../report",
        "printf",
        "command",
        "report;id",
        "report$(id)",
        "*",
        "report/name",
    ] {
        let mut bad = p.clone();
        bad.names = vec![name.into()];
        assert!(bad.validate().is_err(), "{name}");
    }
}
#[test]
fn literal_path_bytes_do_not_dispatch_shell_or_host_filesystem_fallback() {
    let p = profile();
    for path in ["/host-only/path", "/opt/$(id)/bin", "/opt/name;id/bin"] {
        let o = path_lookup::resolve(&p, Some(path)).unwrap();
        assert!(o.reports.iter().all(|r| r.decision == Decision::NotFound));
        assert!(matches!(o.policy, Policy::Untrusted { .. }));
    }
    let source = include_str!("../src/filesystem/path_lookup.rs");
    assert!(
        !source.contains("std::fs")
            && !source.contains("std::process")
            && !source.contains("std::env")
            && !source.contains("Command::new")
    );
}
#[test]
fn source_shape_validation_is_distinct_from_parsing_and_from_search_policy() {
    let p = profile();
    let raw = serde_json::to_vec(&p).unwrap();
    assert_eq!(Profile::decode(&raw).unwrap(), p);
    let mut bad = p.clone();
    bad.names.push(bad.names[0].clone());
    let parsed: Profile = serde_json::from_slice(&serde_json::to_vec(&bad).unwrap()).unwrap();
    assert!(
        parsed.validate().is_err(),
        "duplicate names are semantic source errors"
    );
    let mut bad = p.clone();
    bad.tree.files.values_mut().next().unwrap().push('x');
    assert!(bad.tree.valid());
    assert!(bad.validate().is_err());
    let mut bad = p.clone();
    for n in 0..65 {
        bad.tree
            .nodes
            .insert(format!("/n{n}"), node(true, 0o555, "root", "root"));
    }
    assert!(bad.tree.valid());
    assert!(bad.validate().is_err());
    let mut bad = p.clone();
    bad.tree.user = "root".into();
    assert!(bad.validate().is_err());
    let mut bad = p.clone();
    bad.trusted_directories.push("/absent".into());
    assert!(bad.validate().is_err());
    let mut no_trusted = p.clone();
    no_trusted.trusted_directories.clear();
    no_trusted.validate().unwrap();
    let o = path_lookup::resolve(&no_trusted, Some("/opt/new/bin:/usr/bin")).unwrap();
    assert_eq!(selected(&o, 0), "/opt/new/bin/report");
    assert!(matches!(o.policy, Policy::Untrusted { .. }));
}
#[test]
fn exact_source_equality_refuses_a_valid_forged_inventory_or_trust_roster() {
    let p = profile();
    assert!(p.same_source(&p));
    let mut changed = p.clone();
    changed
        .tree
        .nodes
        .get_mut("/opt/new/bin/report")
        .unwrap()
        .mode = 0o555;
    changed.validate().unwrap();
    assert!(!changed.same_source(&p));
    let mut changed = p.clone();
    changed.trusted_directories.push("/opt/old/bin".into());
    changed.validate().unwrap();
    assert!(!changed.same_source(&p));
}
#[test]
fn new_json_entrypoints_reject_duplicate_unknown_excess_and_oversized_documents() {
    let p = profile();
    let raw = serde_json::to_string(&p).unwrap();
    assert!(
        Profile::decode(
            raw.replace("\"version\":1", "\"version\":1,\"version\":1")
                .as_bytes()
        )
        .is_err()
    );
    assert!(Profile::decode(raw.replacen("{", "{\"unknown\":0,", 1).as_bytes()).is_err());
    assert!(Profile::decode(&vec![b' '; path_lookup::MAX_PROFILE + 1]).is_err());
    let mut v = serde_json::to_value(&p).unwrap();
    v["names"] = serde_json::json!(["a", "b", "c", "d", "e"]);
    assert!(Profile::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut v = serde_json::to_value(&p).unwrap();
    v["names"] = serde_json::json!(["x".repeat(49)]);
    assert!(Profile::decode(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn saved_observation_decode_is_bounded_and_recomputes_actual_decisions_not_pass_flags() {
    let p = profile();
    let o = path_lookup::resolve(&p, Some("/opt/new/bin:/usr/bin")).unwrap();
    assert!(o.matches(&p));
    assert_eq!(
        Observation::decode(&serde_json::to_vec(&o).unwrap(), &p).unwrap(),
        o
    );
    let mut forged = o.clone();
    forged.reports[0].decision = Decision::Selected {
        path: "/opt/old/bin/report".into(),
    };
    assert!(!forged.matches(&p));
    assert!(Observation::decode(&serde_json::to_vec(&forged).unwrap(), &p).is_err());
    let mut forged = o.clone();
    forged.steps = 0;
    assert!(Observation::decode(&serde_json::to_vec(&forged).unwrap(), &p).is_err());
    let unsafe_o = path_lookup::resolve(&p, Some("/opt/new/bin:/usr/bin:/opt/old/bin")).unwrap();
    let mut forged = unsafe_o;
    forged.policy = Policy::Trusted;
    assert!(Observation::decode(&serde_json::to_vec(&forged).unwrap(), &p).is_err());
    assert!(Observation::decode(&vec![b' '; path_lookup::MAX_OBSERVATION + 1], &p).is_err());
}
#[test]
fn old_access_wire_names_stay_exact_and_execute_file_is_only_a_permission_query() {
    for (operation, word) in [
        (Access::ReadFile, "read_file"),
        (Access::WriteFile, "write_file"),
        (Access::ListNames, "list_names"),
        (Access::SearchDirectory, "search_directory"),
        (Access::CreateEntry, "create_entry"),
    ] {
        assert_eq!(
            serde_json::to_string(&operation).unwrap(),
            format!("\"{word}\"")
        );
    }
    assert_eq!(
        serde_json::to_string(&Access::ExecuteFile).unwrap(),
        "\"execute_file\""
    );
    let p = profile();
    let before = p.tree.clone();
    assert!(p.tree.access("alex", "/search/report", Access::ExecuteFile));
    assert_eq!(p.tree, before);
}

#[test]
fn trusted_readonly_directories_reject_current_caller_write_using_owner_class() {
    let p = profile();
    let mut owner = p.clone();
    *owner.tree.nodes.get_mut("/opt/new/bin").unwrap() = node(true, 0o755, "alex", "staff");
    assert!(owner.tree.valid());
    assert!(
        owner.validate().is_err(),
        "owner write is not readonly despite other-class555"
    );
    let mut group = p.clone();
    *group.tree.nodes.get_mut("/opt/new/bin").unwrap() = node(true, 0o575, "root", "ops");
    assert!(group.tree.valid());
    assert!(
        group.validate().is_err(),
        "caller's group write rejects supplied trust"
    );
    let mut owner = p.clone();
    *owner.tree.nodes.get_mut("/opt/new/bin").unwrap() = node(true, 0o577, "alex", "staff");
    owner.validate().unwrap();
    assert_eq!(
        path_lookup::resolve(&owner, Some("/opt/new/bin:/usr/bin"))
            .unwrap()
            .policy,
        Policy::Trusted
    );
    assert!(
        !owner
            .tree
            .access("alex", "/opt/new/bin", Access::CreateEntry),
        "owner class takes precedence over writable other bits"
    );
    let mut denied = p;
    denied.names = vec!["report".into()];
    denied.validate().unwrap();
    assert_eq!(
        path_lookup::resolve(&denied, Some("/denied:/opt/new/bin"))
            .unwrap()
            .reports[0]
            .candidates[0]
            .state,
        CandidateState::Denied,
        "readonly but nonsearchable candidates can be skipped honestly"
    );
}
