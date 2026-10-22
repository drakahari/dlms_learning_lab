use dlms_learning_lab::shell::{self, Case, Vm};
fn fixture() -> Case {
    serde_json::from_str(r#"{"name":"Loop inputs","args":[],"stdin":"before\nafter\nlast","env":{},"files":{"/work/outer file":"red\n\nblue","/work/inner":"one\ntwo\n","/work/other":"separate\n"},"stdout":"","stderr":"","status":0,"after":{}}"#).unwrap()
}
fn output(src: &str, want: &str) {
    let r = shell::run(src, &fixture());
    assert_eq!(r.error, None, "{src}");
    assert_eq!(r.stdout, want, "{src}");
    assert_eq!(r.stderr, "");
    assert_eq!(r.status, 0);
    assert_eq!(r.files, fixture().files);
}
#[test]
fn sequential_input_is_opened_once_and_preserves_the_prior_cursor_and_variables() {
    output(
        "read -r first\nprintf '%s\\n' \"$first\"\np='outer file'\nn=0\nwhile read -r row || [ -n \"$row\" ]; do p=other; n=$((n+1)); printf '%s:%s\\n' \"$n\" \"$row\"; done < \"$p\"\nread -r next\nprintf '%s:%s:%s\\n' \"$next\" \"$p\" \"$n\"",
        "before\n1:red\n2:\n3:blue\nafter:other:3\n",
    );
}
#[test]
fn nested_loop_inputs_restore_the_outer_cursor_and_parent_standard_input() {
    output(
        "while read -r a || [ -n \"$a\" ]; do printf 'A:%s\\n' \"$a\"; while read -r b; do printf 'B:%s\\n' \"$b\"; done < inner; done < 'outer file'\nread -r first\nprintf '%s\\n' \"$first\"",
        "A:red\nB:one\nB:two\nA:\nB:one\nB:two\nA:blue\nB:one\nB:two\nbefore\n",
    );
}
#[test]
fn early_function_return_unwinds_only_its_loop_inputs_even_with_command_redirection() {
    for suffix in ["", " < other"] {
        output(
            &format!(
                "f() {{ while read -r x; do printf 'F:%s\\n' \"$x\"; return 7{suffix}; done < inner; }}\nwhile read -r line || [ -n \"$line\" ]; do printf 'L:%s\\n' \"$line\"; f; printf 'Status:%s\\n' \"$?\"; done < 'outer file'\nread -r first\nprintf '%s\\n' \"$first\""
            ),
            "L:red\nF:one\nStatus:7\nL:\nF:one\nStatus:7\nL:blue\nF:one\nStatus:7\nbefore\n",
        );
    }
    for body in ["return 7 < other", "while true; do return 7 < other; done"] {
        output(
            &format!("f() {{ {body}; }}; f; read -r next; printf '<%s>\\n' \"$next\""),
            "<before>\n",
        );
        output(
            &format!(
                "f() {{ {body}; }}; while read -r row; do f; printf '<%s>\\n' \"$row\"; done < inner; read -r next; printf '<%s>\\n' \"$next\""
            ),
            "<one>\n<two>\n<before>\n",
        );
    }
    output(
        "f() { read -r a; printf 'Fn:%s\\n' \"$a\"; }\nwhile read -r row; do printf 'Row:%s\\n' \"$row\"; f; done < inner\nread -r first\nprintf '%s\\n' \"$first\"",
        "Row:one\nFn:two\nbefore\n",
    );
}
#[test]
fn empty_missing_and_last_body_status_remain_meaningful_and_retryable() {
    let mut c = fixture();
    c.files.insert("/work/empty".into(), String::new());
    let r = shell::run(
        "while read -r a; do false; done < empty\nprintf '%s\\n' \"$?\"\nwhile read -r a; do false; done < inner\nprintf '%s\\n' \"$?\"",
        &c,
    );
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "0\n1\n");
    let r = shell::run(
        "while true; do printf impossible; done < missing || printf 'handled:%s\\n' \"$?\"\nread -r a\nprintf '%s\\n' \"$a\"",
        &c,
    );
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "handled:1\nbefore\n");
    assert_eq!(r.stderr, "shell: missing: No such file\n");
}
#[test]
fn unsupported_redirection_and_active_source_mutation_fail_before_writing() {
    for tail in [
        " > out",
        " < inner < other",
        " < inner > out",
        " < inner 2> out",
    ] {
        let r = shell::run(&format!("while false; do :; done{tail}"), &fixture());
        assert!(r.error.unwrap().contains("Unsupported loop redirection"));
        assert_eq!(r.files, fixture().files);
    }
    for cmd in [
        "printf changed > 'outer file'",
        "printf changed >> /work/outer\\ file",
        "f",
        ": > inner",
    ] {
        let src = format!(
            "f() {{ printf changed > 'outer file'; }}\nwhile read -r a; do while read -r b; do {cmd}; done < inner; done < 'outer file'"
        );
        let r = shell::run(&src, &fixture());
        assert!(
            r.error
                .unwrap()
                .contains("Unsupported write to an active loop input"),
            "{src}"
        );
        assert_eq!(r.files, fixture().files);
    }
    output(
        "while read -r a; do :; done < inner\nread -r a\nprintf '%s\\n' \"$a\"",
        "before\n",
    );
    let r = shell::run("while false; do :; done < ../personal", &fixture());
    assert!(r.error.unwrap().contains("Only ordinary virtual paths"));
}
#[test]
fn stop_instruction_and_snapshot_memory_limits_bound_file_loops() {
    let c = fixture();
    let mut vm = Vm::new("while true; do :; done < inner", &c).unwrap();
    vm.tick(16);
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    assert!(vm.outcome.error.unwrap().starts_with("Stopped"));
    let r = shell::run("while true; do :; done < inner", &c);
    assert!(r.error.unwrap().contains("Execution limit"));
    let mut c = c;
    c.files.insert("/work/big".into(), "x".repeat(60_000));
    let r = shell::run("f() { while true; do f; done < big; }; f", &c);
    assert!(r.error.unwrap().contains("memory"));
    assert_eq!(r.files, c.files);
}
