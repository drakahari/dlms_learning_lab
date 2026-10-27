//! Root capability probes for reviewed next Bash unit. In-process VM only.
use dlms_learning_lab::shell::{self, Case};
use std::collections::BTreeMap;

fn case(args: &[&str]) -> Case {
    Case {
        name: "command vectors preserve literal data".into(),
        args: args.iter().map(|s| s.to_string()).collect(),
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::from([("/work/keep.txt".into(), "original\n".into())]),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}

#[test]
fn array_command_head_preserves_empty_spaces_globs_and_hostile_looking_data() {
    for value in [
        "two words",
        "",
        "*.txt",
        "$(printf injected); > keep.txt",
        "a'\"\\b",
    ] {
        let c = case(&[value]);
        let r = shell::run("command=(printf '<%s>\\n' \"$1\"); \"${command[@]}\"", &c);
        assert_eq!(r.error, None, "{value:?}: {r:?}");
        assert_eq!(r.status, 0);
        assert_eq!(r.stdout, format!("<{value}>\n"));
        assert_eq!(r.files, c.files);
    }
}

#[test]
fn prior_array_replacement_adds_optional_arguments_without_flattening() {
    let source = r#"
command=(printf '%s\n' ready)
if test "$1" = publish; then command=("${command[@]}" "$2"); fi
"${command[@]}" > "$3"
"#;
    for (mode, value, expected) in [
        ("preview", "ignored", "ready\n"),
        ("publish", "two words", "ready\ntwo words\n"),
        ("publish", "", "ready\n\n"),
        ("publish", "$(cat keep.txt)", "ready\n$(cat keep.txt)\n"),
    ] {
        let c = case(&[mode, value, "two words.txt"]);
        let r = shell::run(source, &c);
        assert_eq!(r.error, None, "{r:?}");
        assert_eq!(r.status, 0);
        assert_eq!(r.files["/work/two words.txt"], expected);
        assert_eq!(r.files["/work/keep.txt"], "original\n");
        assert!(r.stdout.is_empty());
    }
}

#[test]
fn quoted_command_vectors_dispatch_supported_functions_and_propagate_their_status() {
    let source = r#"
show() { printf 'item:%s\n' "$1"; return 7; }
command=(show "$1")
"${command[@]}"
printf 'status:%s\n' "$?"
"#;
    let c = case(&["*.txt"]);
    let r = shell::run(source, &c);
    assert_eq!(r.error, None, "{r:?}");
    assert_eq!(r.stdout, "item:*.txt\nstatus:7\n");
    assert_eq!(r.status, 0);
    assert_eq!(r.files, c.files);
}

#[test]
fn explicit_case_dispatch_rejects_unlisted_actions_without_writing() {
    let source = r#"
case "$1" in
  preview) command=(printf '%s\n' "$2");;
  publish) command=(printf '%s\n' "$2"); "${command[@]}" > report.txt; exit 0;;
  *) printf 'Unknown action\n' >&2; exit 2;;
esac
"${command[@]}"
"#;
    for (action, stdout, stderr, status) in [
        ("preview", "literal\n", "", 0),
        ("publish", "", "", 0),
        ("$(printf publish)", "", "Unknown action\n", 2),
    ] {
        let c = case(&[action, "literal"]);
        let r = shell::run(source, &c);
        assert_eq!(r.error, None, "{r:?}");
        assert_eq!(r.stdout, stdout);
        assert_eq!(r.stderr, stderr);
        assert_eq!(r.status, status);
        if action == "publish" {
            assert_eq!(r.files["/work/report.txt"], "literal\n");
        } else {
            assert_eq!(r.files, c.files);
        }
    }
}

#[test]
fn expanded_control_operator_is_a_command_name_and_is_never_reparsed_as_shell_syntax() {
    for head in ["/bin/sh", "printf;", "eval", "source", "$(printf printf)"] {
        let c = case(&[head]);
        let r = shell::run("command=(\"$1\" injected); \"${command[@]}\"", &c);
        assert!(
            r.error
                .as_ref()
                .is_some_and(|s| s.to_ascii_lowercase().contains("unsupported")),
            "{head}: {r:?}"
        );
        assert!(r.stdout.is_empty());
        assert_eq!(r.files, c.files);
    }
}

#[test]
fn scalar_command_heads_do_not_require_array_syntax_but_keep_argument_boundaries() {
    let c = case(&["two words", ""]);
    let r = shell::run("tool=printf; \"$tool\" '<%s>\\n' \"$1\" \"$2\"", &c);
    assert_eq!(r.error, None, "{r:?}");
    assert_eq!(r.stdout, "<two words>\n<>\n");
    assert_eq!(r.files, c.files);
}
