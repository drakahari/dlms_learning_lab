//! Flat simulated utilities: actual evaluation, boundaries and active descriptors.
use dlms_learning_lab::shell::{self, Case, Vm};
use std::collections::BTreeMap;
fn case(files: &[(&str, &str)]) -> Case {
    let files = files
        .iter()
        .map(|(p, t)| (p.to_string(), t.to_string()))
        .collect();
    Case {
        name: "virtual-file fixture".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files,
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn run(source: &str, files: &[(&str, &str)]) -> shell::Outcome {
    shell::run(source, &case(files))
}
#[test]
fn temporary_capture_write_publish_and_remove_operate_on_parent_visible_virtual_files() {
    let r = run(
        "tmp=$(mktemp -- /work/report.XXXXXX)\nprintf '%s\\n' ready > \"$tmp\"\nmv -- \"$tmp\" /work/report.txt\ncat /work/report.txt",
        &[("/work/keep.txt", "keep")],
    );
    assert!(r.error.is_none(), "{r:?}");
    assert_eq!((r.stdout.as_str(), r.status), ("ready\n", 0));
    assert_eq!(
        r.files,
        BTreeMap::from([
            ("/work/keep.txt".into(), "keep".into()),
            ("/work/report.txt".into(), "ready\n".into())
        ])
    );
}
#[test]
fn sequence_and_file_effects_are_shared_across_captures_and_collision_aware() {
    let r = run(
        "a=$(mktemp /work/name.XXXXXX)\nb=$(mktemp /work/other.XXXXXX)\nprintf '%s|%s\\n' \"$a\" \"$b\"\nrm \"$a\"\nrm \"$b\"",
        &[("/work/name.000000", "preserve collision")],
    );
    assert!(r.error.is_none(), "{r:?}");
    assert_eq!(r.stdout, "/work/name.000001|/work/other.000002\n");
    assert_eq!(
        r.files,
        BTreeMap::from([("/work/name.000000".into(), "preserve collision".into())])
    );
    let r = run(
        "t=$(mktemp relative.XXXXXX); printf '%s\\n' \"$t\"; rm -- \"$t\"",
        &[],
    );
    assert!(r.error.is_none(), "{r:?}");
    assert_eq!(r.stdout, "relative.000000\n");
    assert!(r.files.is_empty());
}
#[test]
fn supported_missing_and_same_file_failures_allow_normal_status_recovery() {
    let r = run(
        "rm /work/missing || printf '%s\\n' recovered\nmv /work/a a || printf '%s\\n' unchanged\nmv /work/missing /work/a || printf '%s\\n' retained",
        &[("/work/a", "original")],
    );
    assert!(r.error.is_none(), "{r:?}");
    assert_eq!(r.status, 0);
    assert_eq!(r.stdout, "recovered\nunchanged\nretained\n");
    assert!(r.stderr.contains("No such virtual file") && r.stderr.contains("same virtual file"));
    assert_eq!(r.files["/work/a"], "original");
}
#[test]
fn quoted_names_terminator_and_replacing_a_regular_destination_work() {
    let r = run(
        "mv -- 'source name' 'destination name'\nrm -- --\ncat 'destination name'",
        &[
            ("/work/source name", "new"),
            ("/work/destination name", "old"),
            ("/work/--", "remove"),
        ],
    );
    assert!(r.error.is_none(), "{r:?}");
    assert_eq!(r.stdout, "new");
    assert_eq!(
        r.files,
        BTreeMap::from([("/work/destination name".into(), "new".into())])
    );
}
#[test]
fn unsupported_options_templates_traversal_and_directories_refuse_honestly() {
    for source in [
        "mktemp",
        "mktemp /work/x.XXX",
        "mktemp /work/x.XXXXXXX",
        "mktemp -d /work/x.XXXXXX",
        "mktemp /tmp/x.XXXXXX",
        "mktemp ../x.XXXXXX",
        "rm -f /work/a",
        "rm /work/a /work/b",
        "rm /work/sub/a",
        "mv -T /work/a /work/b",
        "mv /work/a ../b",
    ] {
        let r = run(source, &[("/work/a", "protected")]);
        assert!(r.error.is_some(), "{source}: {r:?}");
        assert_eq!(r.status, 2);
        assert_eq!(r.files["/work/a"], "protected");
        assert_eq!(r.files.len(), 1);
    }
}
#[test]
fn current_stdout_stderr_and_normalized_alias_endpoints_refuse_after_redirection_effects() {
    for source in [
        "rm a > /work/a",
        "rm /work/a 2> a",
        "mv /work/a /work/b > /work/a",
        "mv /work/b /work/a > /work/a",
    ] {
        let r = run(source, &[("/work/a", "a bytes"), ("/work/b", "b bytes")]);
        assert!(
            r.error
                .as_ref()
                .is_some_and(|e| e.contains("active output")),
            "{source}: {r:?}"
        );
        assert_eq!(r.files["/work/a"], "");
        assert_eq!(r.files["/work/b"], "b bytes");
    }
}
#[test]
fn suspended_open_outputs_are_protected_but_future_redirect_words_are_not_descriptors() {
    for source in [
        "printf '%s' x > /work/a > \"$(rm /work/a)\"",
        "printf '%s' x > /work/a 2> \"$(printf '%s' \"$(rm a)\")\"",
    ] {
        let r = run(source, &[("/work/a", "original")]);
        assert!(
            r.error
                .as_ref()
                .is_some_and(|e| e.contains("active output")),
            "{source}: {r:?}"
        );
        assert_eq!(r.files["/work/a"], "");
    }
    let r = run(
        "printf '%s' \"$(rm /work/a)\" > /work/a",
        &[("/work/a", "original")],
    );
    assert!(r.error.is_none(), "{r:?}");
    assert_eq!(r.files["/work/a"], "");
    let r = run(
        "printf '%s' \"$(printf '%s' \"$(rm /work/a)\")\" > /work/a",
        &[("/work/a", "original")],
    );
    assert!(r.error.is_none(), "{r:?}");
    assert_eq!(r.files["/work/a"], "");
}
#[test]
fn active_loop_input_cannot_be_removed_or_used_as_either_rename_endpoint() {
    for command in ["rm /work/a", "mv /work/a /work/b", "mv /work/b /work/a"] {
        let r = run(
            &format!("while IFS= read -r row; do {command}; done < /work/a"),
            &[("/work/a", "one\ntwo\n"), ("/work/b", "retain")],
        );
        assert!(
            r.error
                .as_ref()
                .is_some_and(|e| e.contains("active") && e.contains("input")),
            "{command}: {r:?}"
        );
        assert_eq!(r.files["/work/a"], "one\ntwo\n");
        assert_eq!(r.files["/work/b"], "retain");
    }
}
#[test]
fn file_limit_instruction_budget_and_stop_bound_repeated_mutation() {
    let mut c = case(&[]);
    for n in 0..64 {
        c.files.insert(format!("/work/existing{n}"), String::new());
    }
    let r = shell::run("mktemp /work/t.XXXXXX", &c);
    assert!(r.error.as_ref().is_some_and(|e| e.contains("file limit")));
    assert_eq!(r.files, c.files);
    let mut vm = Vm::new(
        "while true; do t=$(mktemp /work/t.XXXXXX); rm \"$t\"; done",
        &case(&[]),
    )
    .unwrap();
    for _ in 0..20 {
        vm.tick(4);
    }
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    assert!(vm.outcome.error.as_ref().unwrap().contains("Stopped"));
    let r = run(
        "while true; do t=$(mktemp /work/t.XXXXXX); rm \"$t\"; done",
        &[],
    );
    assert!(
        r.error.as_ref().is_some_and(|e| e.contains("limit")),
        "{r:?}"
    );
    assert!(r.files.len() <= 1);
}
#[test]
fn ordinary_functions_named_after_external_utilities_keep_their_existing_precedence() {
    let r = run(
        "rm() { printf '%s\\n' helper; }\nrm /work/a",
        &[("/work/a", "retain")],
    );
    assert!(r.error.is_none(), "{r:?}");
    assert_eq!(r.stdout, "helper\n");
    assert_eq!(r.files["/work/a"], "retain");
}

#[test]
fn ordinary_input_redirects_and_memory_preflight_preserve_sources() {
    for command in ["rm a", "mv a b", "mv b a"] {
        let r = run(
            &format!("{command} < /work/a"),
            &[("/work/a", "input\n"), ("/work/b", "retain")],
        );
        assert!(
            r.error.as_ref().is_some_and(|e| e.contains("active input")),
            "{command}: {r:?}"
        );
        assert_eq!(r.files["/work/a"], "input\n");
        assert_eq!(r.files["/work/b"], "retain");
    }
    for command in ["mktemp new.XXXXXX", "mv a b", "rm a"] {
        let mut c = case(&[]);
        c.files.insert("/work/a".into(), "x".repeat(129000));
        c.env.insert("PAD".into(), "x".repeat(512));
        assert!(c.valid());
        let source = format!(
            "i=0; while [ \"$i\" -lt 256 ]; do printf '%s' \"$PAD\" >> a; i=$((i+1)); done\n{command}"
        );
        let r = shell::run(&source, &c);
        assert!(
            r.error.as_ref().is_some_and(|e| e.contains("memory")),
            "{command}: {:?}",
            r.error
        );
        assert_eq!(r.files.len(), 1, "No new/moved file on memory refusal");
        assert_eq!(
            r.files["/work/a"],
            "x".repeat(260072),
            "Setup completes, then utility refuses before mutation"
        );
    }
}
