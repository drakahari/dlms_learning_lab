use dlms_learning_lab::shell::{self, Case, Outcome, Vm};
use std::collections::BTreeMap;
fn fixture(input: &str) -> Case {
    Case {
        name: "Text utilities".into(),
        args: vec![],
        stdin: input.into(),
        env: BTreeMap::new(),
        files: BTreeMap::new(),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn run(source: &str, input: &str) -> Outcome {
    shell::run(source, &fixture(input))
}
fn expect(source: &str, input: &str, output: &str) {
    let r = run(source, input);
    assert_eq!(r.error, None, "{source}: {:?}", r.error);
    assert_eq!(r.stdout, output, "{source}");
    assert_eq!(r.stderr, "");
    assert_eq!(r.status, 0);
}
#[test]
fn sort_uses_whole_lines_fixed_byte_order_uniqueness_and_reverse_without_numeric_guessing() {
    expect("sort", "b\nA\na\nb\n2\n10\n\n", "\n10\n2\nA\na\nb\nb\n");
    expect("sort -ru", "b\na\nb\na\n", "b\na\n");
    expect("sort -u -r", "é\nz\nÉ\nz", "é\nÉ\nz\n");
    expect("sort", "\n\r\n\n", "\n\n\r\n");
    for source in ["sort", "sort -u", "sort -r"] {
        expect(source, "", "");
        expect(source, "last", "last\n");
    }
}
#[test]
fn uniq_collapses_only_adjacent_groups_with_correct_counts_and_filters() {
    expect("uniq", "a\na\nb\na\na", "a\nb\na\n");
    expect(
        "uniq -c",
        "a\na\nb\na\na",
        "      2 a\n      1 b\n      2 a\n",
    );
    expect("uniq -d", "a\na\nb\nc\nc\n", "a\nc\n");
    expect("uniq -cu", "a\na\nb\nc\nc\n", "      1 b\n");
    expect("uniq -cd", "\n\nlast", "      2 \n");
    expect("uniq", "", "");
    expect("uniq -u", "one", "one\n");
    expect("uniq -c", "\r\n\r\n", "      2 \r\n");
}
#[test]
fn cut_preserves_input_order_empty_fields_and_delimiter_free_lines() {
    expect(
        "cut -d : -f 3,1-2,2,8",
        "a:b:c:d\nsolo\n:a::\n",
        "a:b:c\nsolo\n:a:\n",
    );
    expect(
        "cut -s -d : -f 3,1-2,2",
        "a:b:c:d\nsolo\n\n:a::\n",
        "a:b:c\n:a:\n",
    );
    expect("cut -d : -f 5", "a:b\nsolo\n:a\n", "\nsolo\n\n");
    expect("cut -d : -f 2-", "a:b:c\nx::\nlast:", "b:c\n:\n\n");
    expect("cut -d : -f -2", "a:b:c\n:a:\n", "a:b\n:a\n");
    expect("cut -f 2", "a\tb\tc\nsolo\n", "b\nsolo\n");
    expect("cut -d : -f 1", "é:z\n", "é\n");
    expect("cut -d : -f 1", "", "");
    expect("cut -d : -f 1", "\n", "\n");
    expect("cut -s -d : -f 1", "\n", "");
}
#[test]
fn virtual_file_stdin_pipeline_redirection_and_function_overrides_remain_isolated() {
    let mut c = fixture("z\nx\nz\n");
    c.files.insert(
        "/work/data".into(),
        "beta:desk\nalpha:router\nbeta:desk\n".into(),
    );
    c.files.insert("/work/-u".into(), "one\none\n".into());
    let r = shell::run(
        "cut -d : -f 1 data | sort | uniq -c > report\nsort -u -\nuniq -- -u",
        &c,
    );
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "x\nz\none\n");
    assert_eq!(r.files["/work/report"], "      1 alpha\n      2 beta\n");
    assert_eq!(r.files["/work/data"], c.files["/work/data"]);
    // sort/uniq/cut are utilities, not shell builtins: existing functions can override them.
    expect("sort() { printf 'helper\\n'; }\nsort", "", "helper\n");
    let missing = run("cut -d : -f 1 < data", "");
    assert_eq!(missing.status, 1);
    assert!(missing.stderr.contains("No such file"));
}
#[test]
fn malformed_and_unsupported_options_are_honest_and_paths_never_escape() {
    for source in [
        "sort -n",
        "sort -o /tmp/host",
        "sort a b",
        "sort data -r",
        "uniq -du",
        "uniq -i",
        "uniq +10",
        "uniq a b",
        "cut -b 1",
        "cut -d : -f1",
        "cut -d é -f 1",
        "cut -d '' -f 1",
        "cut -d '\n' -f 1",
        "cut -f 1000001",
        "cut -f 9999999999999999999999999",
    ] {
        let r = run(source, "");
        assert!(
            r.error.as_ref().is_some_and(|e| e.contains("Unsupported")),
            "{source}: {r:?}"
        );
        assert!(r.stdout.is_empty());
    }
    for source in [
        "cut",
        "cut -f 0",
        "cut -f 2-1",
        "cut -f 1,,2",
        "cut -f -",
        "cut -f 1-x",
        "cut -f",
        "cut -d",
        "cut -f +1",
    ] {
        let r = run(source, "");
        assert!(
            r.error.as_ref().is_some_and(|e| e.starts_with("cut:")),
            "{source}: {r:?}"
        );
    }
    for source in [
        "sort /etc/passwd",
        "uniq ../outside",
        "cut -f 1 /work/../../outside",
        "sort -- ../outside",
    ] {
        let r = run(source, "");
        assert!(r.error.is_some(), "{source}: {r:?}");
        assert!(r.files.is_empty());
    }
}
#[test]
fn missing_files_return_realistic_status_and_messages_and_locale_changes_are_rejected() {
    for (cmd, status, message) in [
        (
            "sort",
            2,
            "sort: cannot read: missing: No such file or directory\n",
        ),
        ("uniq", 1, "uniq: missing: No such file or directory\n"),
        ("cut -f 1", 1, "cut: missing: No such file or directory\n"),
    ] {
        let r = run(&format!("{cmd} missing"), "untouched");
        assert_eq!(r.error, None);
        assert_eq!(r.status, status);
        assert_eq!(r.stderr, message);
    }
    for cmd in ["sort", "uniq", "cut -f 1"] {
        let mut c = fixture("b\na\n");
        c.env.insert("LANG".into(), "sv_SE.UTF-8".into());
        assert!(
            shell::run(cmd, &c)
                .error
                .unwrap()
                .contains("Unsupported text-utility locale")
        );
        c.env.insert("LC_ALL".into(), "C".into());
        assert!(shell::run(cmd, &c).error.is_none());
        c.env.insert("LC_ALL".into(), "C.UTF-8".into());
        assert!(shell::run(cmd, &c).error.is_some());
    }
}
#[test]
fn line_range_output_instruction_and_stop_limits_stay_bounded() {
    for cmd in ["sort -u", "uniq", "cut -d : -f 1"] {
        assert!(run(cmd, &"x\n".repeat(8192)).error.is_none());
        let r = run(cmd, &"x\n".repeat(8193));
        assert!(r.error.unwrap().contains("8192 lines"));
    }
    let fields = (1..=129)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");
    assert!(
        run(&format!("cut -f {fields}"), "")
            .error
            .unwrap()
            .contains("128 ranges")
    );
    let input = "a".repeat(32768);
    assert!(run("sort", &input).error.unwrap().contains("output limit"));
    let mut vm = Vm::new("while true; do sort -u; done", &fixture("b\na\n")).unwrap();
    vm.tick(2);
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    assert!(vm.outcome.error.unwrap().contains("Stopped"));
    assert!(
        run("while true; do cut -f 1; done", "")
            .error
            .unwrap()
            .contains("limit")
    );
}

#[test]
fn pipelines_preserve_first_stage_input_consumption_but_isolate_variables_and_redirects() {
    expect(
        "sort | uniq\nread -r next\nprintf 'after:%s:<%s>\\n' \"$?\" \"$next\"",
        "b\na\nb\n",
        "a\nb\nafter:1:<>\n",
    );
    expect(
        "read -r item | sort\nread -r next\nprintf 'after:%s:<%s>:<%s>\\n' \"$?\" \"$next\" \"$item\"",
        "first\nsecond\n",
        "after:0:<second>:<>\n",
    );
    expect(
        "printf 'x\\n' | sort\nread -r next\nprintf '<%s>\\n' \"$next\"",
        "keep\n",
        "x\n<keep>\n",
    );
    let mut c = fixture("keep\nnext\n");
    c.files.insert("/work/data".into(), "z\na\n".into());
    for first in ["sort data", "sort < data"] {
        let r = shell::run(
            &format!("{first} | uniq\nread -r next\nprintf '<%s>\\n' \"$next\""),
            &c,
        );
        assert_eq!(r.error, None);
        assert_eq!(r.stdout, "a\nz\n<keep>\n");
    }
    expect(
        "read -r ignored\ngrep -F a | sort\nread -r next\nprintf 'status:%s:<%s>\\n' \"$?\" \"$next\"",
        "skip\na\nb\n",
        "a\nstatus:1:<>\n",
    );
}
