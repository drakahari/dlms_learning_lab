//! Proposed adapter tests only: in-process VM plus readonly typed lookup.
use dlms_learning_lab::{
    engine::Node,
    environment_workspace as e,
    filesystem::{Tree, path_lookup as core},
    shell,
};
use e::path_lookup::{ErrorKind, Expected, Plan, Run};
use std::collections::BTreeMap;
fn node(directory: bool, mode: u16) -> Node {
    Node {
        directory,
        mode,
        owner: "root".into(),
        group: "root".into(),
    }
}
fn task() -> e::Task {
    let mut nodes = BTreeMap::new();
    let mut files = BTreeMap::new();
    for name in [
        "/",
        "/home",
        "/home/alex",
        "/work",
        "/usr",
        "/usr/bin",
        "/opt",
        "/opt/retired",
        "/opt/retired/bin",
        "/opt/extra",
        "/opt/extra/bin",
        "/denied",
        "/denied/bin",
    ] {
        nodes.insert(name.into(), node(true, 0o555));
    }
    nodes.get_mut("/home/alex").unwrap().owner = "alex".into();
    nodes.get_mut("/home/alex").unwrap().group = "staff".into();
    nodes.get_mut("/denied").unwrap().mode = 0o700;
    let mut trusted = vec![
        "/usr/bin".into(),
        "/opt/extra/bin".into(),
        "/denied/bin".into(),
    ];
    for n in 0..6 {
        nodes.insert(format!("/opt/tool{n}"), node(true, 0o555));
        nodes.insert(format!("/opt/tool{n}/bin"), node(true, 0o555));
        let file = format!("/opt/tool{n}/bin/reportgen");
        nodes.insert(file.clone(), node(false, 0o111));
        files.insert(file, String::new());
        trusted.push(format!("/opt/tool{n}/bin"));
    }
    for (file, mode) in [
        ("/usr/bin/reportgen", 0o644),
        ("/usr/bin/auditprobe", 0o555),
        ("/opt/retired/bin/reportgen", 0o555),
        ("/opt/extra/bin/reportgen", 0o555),
        ("/denied/bin/reportgen", 0o555),
    ] {
        nodes.insert(file.into(), node(false, mode));
        files.insert(file.into(), String::new());
    }
    let tree = Tree {
        nodes,
        files,
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into()]),
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
    let profile = core::Profile {
        version: 1,
        tree,
        trusted_directories: trusted,
        names: vec!["reportgen".into(), "auditprobe".into()],
    };
    profile.validate().unwrap();
    let mut cases = Vec::new();
    let mut expected = Vec::new();
    for n in 0..6 {
        let name = format!("PATH sample {n}");
        let dir = format!("/opt/tool{n}/bin");
        let path = format!("{dir}:/usr/bin");
        let token = format!("synthetic {n} * %");
        // Incoming PATH deliberately absent. A newly assigned private PATH is
        // not already exported; forgetting export is a real observable failure.
        let env = BTreeMap::from([
            ("TOOL_DIR".into(), dir),
            ("BASE_PATH".into(), "/usr/bin".into()),
            ("TOKEN".into(), token.clone()),
        ]);
        let files = BTreeMap::from([("/work/keep.txt".into(), format!("Retain sample {n}\n"))]);
        let input = shell::Case {
            name: name.clone(),
            args: vec![],
            stdin: String::new(),
            env: env.clone(),
            files: files.clone(),
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
            after: files.clone(),
        };
        let mut child_env = env;
        child_env.insert("PATH".into(), path.clone());
        let child = shell::Case {
            name: name.clone(),
            args: vec![],
            stdin: String::new(),
            env: child_env,
            files: files.clone(),
            stdout: format!("Worker token: {token}\n"),
            stderr: String::new(),
            status: 0,
            after: files,
        };
        cases.push(e::Case {
            input,
            parent: BTreeMap::from([(
                "PATH".into(),
                shell::EnvironmentVariable {
                    value: Some(path),
                    exported: true,
                },
            )]),
            child,
        });
        expected.push(Expected {
            case_name: name,
            selected: BTreeMap::from([
                (
                    "reportgen".into(),
                    vec![format!("/opt/tool{n}/bin/reportgen")],
                ),
                ("auditprobe".into(), vec!["/usr/bin/auditprobe".into()]),
            ]),
        });
    }
    let t = e::Task {
        id: "path-assignment".into(),
        instructions: "Repair inherited worker lookup without executing utilities.".into(),
        starter: String::new(),
        child_source: "printf 'Worker token: %s\n' \"$TOKEN\"\n".into(),
        cases,
        path_lookup: Some(Plan {
            profile,
            cases: expected,
        }),
    };
    t.validate().unwrap();
    t
}
const SOURCE: &str = "export PATH=\"$TOOL_DIR:$BASE_PATH\"\n";
fn draft(t: &e::Task, source: &str) -> e::Draft {
    e::Draft {
        source: source.into(),
        result: Some(e::run(t, source).unwrap()),
    }
}
fn passed(t: &e::Task, d: &e::Draft) -> bool {
    e::checks(t, d).iter().all(|r| r.0)
}
fn definition() -> e::Definition {
    let assignment = task();
    let mut example = task();
    example.id = "path-example".into();
    e::Definition {
        id: "path-context".into(),
        assignment,
        example,
    }
}
#[test]
fn actual_six_case_parent_snapshots_feed_lookup_and_fixed_report_remains_separate() {
    let t = task();
    t.validate_child_examples().unwrap();
    let d = draft(&t, SOURCE);
    assert!(passed(&t, &d));
    for (c, r) in t.cases.iter().zip(&d.result.unwrap().results) {
        let snapshot = r.bindings.as_ref().unwrap();
        let Some(Run::Observed { observation }) = &r.path_lookup else {
            panic!()
        };
        assert_eq!(observation.path, snapshot.child["PATH"]);
        assert_eq!(observation.policy, core::Policy::Trusted);
        assert!(
            matches!(&observation.reports[0].decision,core::Decision::Selected{path} if path==&format!("{}/reportgen",c.input.env["TOOL_DIR"]))
        );
        assert_eq!(r.parent.files, c.input.files);
        assert!(c.child.accepts(r.child.as_ref().unwrap()));
    }
}
#[test]
fn canonical_and_equivalent_path_strings_pass_without_waiving_other_environment_fields() {
    let t = task();
    for source in [
        SOURCE,
        "PATH=\"$BASE_PATH:$TOOL_DIR\"\nexport PATH\n",
        "private=$TOOL_DIR\nexport PATH=\"$TOOL_DIR:$TOOL_DIR:$BASE_PATH:/opt/extra/bin\"\n",
    ] {
        assert!(passed(&t, &draft(&t, source)), "{source}");
    }
    for source in [
        "export PATH=\"$TOOL_DIR:$BASE_PATH\"\nexport TOKEN=changed\n",
        "export PATH=\"$TOOL_DIR:$BASE_PATH\"\nexport EXTRA=unrequested\n",
        "export PATH=\"$TOOL_DIR:$BASE_PATH\"\nprintf changed > keep.txt\n",
    ] {
        assert!(!passed(&t, &draft(&t, source)), "{source}");
    }
}
#[test]
fn six_changing_locations_reject_first_case_hardcoding_and_parent_print_imitation() {
    let t = task();
    let d = draft(&t, "export PATH=\"/opt/tool0/bin:$BASE_PATH\"\n");
    let checks = e::checks(&t, &d);
    assert!(checks[0].0);
    assert_eq!(checks.iter().filter(|r| r.0).count(), 1);
    let d = draft(
        &t,
        "printf 'Worker token: %s\n' \"$TOKEN\"\nexport PATH=\"$TOOL_DIR:$BASE_PATH\"\n",
    );
    assert!(!passed(&t, &d));
}
#[test]
fn missing_export_unset_and_unsupported_path_have_real_typed_errors_not_fake_defaults() {
    let t = task();
    for source in [
        "PATH=\"$TOOL_DIR:$BASE_PATH\"\n",
        "export PATH\n",
        "export PATH=\":$BASE_PATH\"\n",
    ] {
        let d = draft(&t, source);
        assert!(!passed(&t, &d));
        assert!(d.result.unwrap().results.iter().all(|r| matches!(
            r.path_lookup,
            Some(Run::Error {
                category: ErrorKind::Unsupported,
                ..
            })
        )));
    }
}
#[test]
fn untrusted_unused_suffix_wrong_provider_and_permission_trace_fail_despite_good_report() {
    let t = task();
    for source in [
        "export PATH=\"$TOOL_DIR:$BASE_PATH:/opt/retired/bin\"\n",
        "export PATH=\"/opt/retired/bin:$TOOL_DIR:$BASE_PATH\"\n",
        "export PATH=\"/denied/bin:$BASE_PATH\"\n",
    ] {
        let d = draft(&t, source);
        assert!(!passed(&t, &d));
        for (c, r) in t.cases.iter().zip(&d.result.unwrap().results) {
            assert!(c.child.accepts(r.child.as_ref().unwrap()));
        }
    }
    let d = draft(&t, "export PATH=\"/denied/bin:$BASE_PATH\"\n");
    assert!(e::checks(&t, &d)[0].1.contains("execute permission"));
}
#[test]
fn exact_case_names_not_expectation_order_bind_selected_paths() {
    let mut t = task();
    t.path_lookup.as_mut().unwrap().cases.reverse();
    t.validate().unwrap();
    assert!(passed(&t, &draft(&t, SOURCE)));
    let mut bad = t.clone();
    bad.path_lookup.as_mut().unwrap().cases[0].case_name = "unknown".into();
    assert!(bad.validate().is_err());
    let mut bad = t;
    let name = bad.path_lookup.as_ref().unwrap().cases[0].case_name.clone();
    bad.path_lookup.as_mut().unwrap().cases[1].case_name = name;
    assert!(bad.validate().is_err());
}
#[test]
fn imported_records_never_keep_runtime_credit_and_wrong_actual_path_metadata_is_rejected() {
    let def = definition();
    let mut s = e::Session::new(def.clone());
    s.assignment.source = SOURCE.into();
    let r = e::run(&def.assignment, SOURCE).unwrap();
    assert!(s.assignment.store(r, &def.assignment));
    assert!(s.solved());
    let raw = serde_json::to_vec(&s).unwrap();
    let restored: e::Session = serde_json::from_slice(&raw).unwrap();
    restored.validate(&def).unwrap();
    assert!(!restored.solved());
    assert!(restored.assignment.needs_reverification(&def.assignment));
    let mut bad = restored.clone();
    let row = &mut bad.assignment.result.as_mut().unwrap().results[0];
    let alternate = def.assignment.path_lookup.as_ref().unwrap().profile.clone();
    row.path_lookup = Some(Run::Observed {
        observation: core::resolve(&alternate, Some("/usr/bin:/opt/tool0/bin")).unwrap(),
    });
    assert!(
        bad.validate(&def).is_err(),
        "valid alternate lookup is forged when it is not the ACTUAL inherited snapshot path"
    );
    let mut bad = restored.clone();
    bad.assignment.result.as_mut().unwrap().results[0].path_lookup = None;
    assert!(bad.validate(&def).is_err());
    let mut bad = restored;
    bad.definition
        .assignment
        .path_lookup
        .as_mut()
        .unwrap()
        .profile
        .tree
        .nodes
        .get_mut("/opt/extra/bin/reportgen")
        .unwrap()
        .mode = 0o111;
    assert!(bad.validate(&def).is_err());
}
#[test]
fn old_none_context_rejects_extra_lookup_metadata_and_serializes_no_new_field() {
    let mut t = task();
    t.path_lookup = None;
    t.validate().unwrap();
    let d = draft(&t, SOURCE);
    assert!(passed(&t, &d));
    let task_json = serde_json::to_value(&t).unwrap();
    assert!(!task_json.as_object().unwrap().contains_key("path_lookup"));
    for r in &d.result.as_ref().unwrap().results {
        assert!(r.path_lookup.is_none());
        assert!(
            !serde_json::to_value(r)
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("path_lookup")
        );
    }
    let mut example = t.clone();
    example.id = "old-example".into();
    let def = e::Definition {
        id: "old-none".into(),
        assignment: t.clone(),
        example,
    };
    let mut s = e::Session::new(def.clone());
    s.assignment = d;
    let newer = draft(&task(), SOURCE);
    s.assignment.result.as_mut().unwrap().results[0].path_lookup =
        newer.result.unwrap().results[0].path_lookup.clone();
    assert!(s.validate(&def).is_err());
}
#[test]
fn source_plan_shape_and_reference_reachability_refuse_malformed_or_unachievable_targets() {
    let t = task();
    let mut bad = t.clone();
    bad.path_lookup.as_mut().unwrap().cases[0]
        .selected
        .get_mut("reportgen")
        .unwrap()[0] = "/usr/bin/reportgen".into();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.path_lookup.as_mut().unwrap().cases[0]
        .selected
        .remove("auditprobe");
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    let extra = bad.path_lookup.as_ref().unwrap().cases[0].clone();
    bad.path_lookup.as_mut().unwrap().cases.push(extra);
    assert!(bad.validate().is_err());
    for extra in [
        "opt/tool0/bin/reportgen",
        "/opt/tool0/bin/./reportgen",
        "/opt/retired/bin/reportgen",
    ] {
        let mut bad = t.clone();
        bad.path_lookup.as_mut().unwrap().cases[0]
            .selected
            .get_mut("reportgen")
            .unwrap()
            .push(extra.into());
        assert!(
            bad.validate().is_err(),
            "every offered alternative must be a canonical source artifact in a trusted direct parent: {extra}"
        );
    }
    let raw = serde_json::to_vec(&t).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    value["path_lookup"]["cases"][0]["selected"]["reportgen"] =
        serde_json::json!(["x".repeat(1025)]);
    assert!(serde_json::from_value::<e::Task>(value).is_err());
}
#[test]
fn stop_in_setup_and_after_lookup_never_publishes_a_complete_record() {
    let t = task();
    for advances in [0, 1, 2, 3] {
        let mut job = e::Job::new(&t, SOURCE).unwrap();
        for _ in 0..advances {
            job.advance(1);
        }
        job.stop();
        assert!(job.done());
        assert!(job.record().is_none());
    }
    let mut job = e::Job::new(&t, "while true; do :; done\n").unwrap();
    for _ in 0..10 {
        job.advance(64);
    }
    job.stop();
    assert!(job.record().is_none());
}
#[test]
fn example_assignment_drafts_reset_and_edit_invalidation_stay_independent() {
    let def = definition();
    let mut s = e::Session::new(def.clone());
    let before = s.assignment.clone();
    s.example.edit(SOURCE.into()).unwrap();
    assert!(
        s.example
            .store(e::run(&def.example, SOURCE).unwrap(), &def.example)
    );
    assert_eq!(s.assignment, before);
    assert!(!s.solved());
    s.assignment.edit(SOURCE.into()).unwrap();
    assert!(
        s.assignment
            .store(e::run(&def.assignment, SOURCE).unwrap(), &def.assignment)
    );
    assert!(s.solved());
    let example = s.example.clone();
    s.assignment
        .edit("export PATH=\"/opt/retired/bin:$BASE_PATH\"\n".into())
        .unwrap();
    assert!(s.assignment.result.is_none());
    assert!(!s.solved());
    assert_eq!(s.example, example);
    let restored = e::Session::new(def);
    assert!(!restored.solved());
    assert_eq!(restored.assignment, before);
}

#[test]
fn capability_fixture_matches_test_constructor_and_old_none_json_has_exact_prior_shape() {
    let t = task();
    let fixture: e::Task =
        serde_json::from_str(include_str!("fixtures/environment-path-lookup.json")).unwrap();
    assert_eq!(t, fixture);
    let mut old = t;
    old.path_lookup = None;
    let expected = format!(
        "{{\"id\":{},\"instructions\":{},\"starter\":{},\"child_source\":{},\"cases\":{}}}",
        serde_json::to_string(&old.id).unwrap(),
        serde_json::to_string(&old.instructions).unwrap(),
        serde_json::to_string(&old.starter).unwrap(),
        serde_json::to_string(&old.child_source).unwrap(),
        serde_json::to_string(&old.cases).unwrap()
    );
    assert_eq!(
        serde_json::to_string(&old).unwrap(),
        expected,
        "old Task field order and bytes gain no optional lookup field"
    );
    let record = e::run(&old, SOURCE).unwrap();
    let row = &record.results[0];
    let expected = format!(
        "{{\"parent\":{},\"bindings\":{},\"child\":{},\"diagnostic\":{}}}",
        serde_json::to_string(&row.parent).unwrap(),
        serde_json::to_string(&row.bindings).unwrap(),
        serde_json::to_string(&row.child).unwrap(),
        serde_json::to_string(&row.diagnostic).unwrap()
    );
    assert_eq!(
        serde_json::to_string(row).unwrap(),
        expected,
        "old CaseRun field order and bytes gain no optional lookup field"
    );
    // Root retains the existing environment target/fixtures to prove historical
    // behavior; this test does not invent a pre-change executed binary identity.
}
