//! Behavioral tracing contract; no host program is executed by these tests.
use dlms_learning_lab::shell::{self, Case, Outcome, Vm};
use std::collections::BTreeMap;
fn case() -> Case {
    Case {
        name: "Tracing contract".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::from([("/work/keep".into(), "retained\n".into())]),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn sliced(source: &str, c: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, c).expect("Supported test program compiles");
    for _ in 0..20000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(slice);
    }
    panic!("Bounded cooperative execution did not finish");
}
#[test]
fn toggles_trace_only_expanded_reached_commands_and_preserve_program_status() {
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    let c = case();
    let source = "set -x\nfalse\nresult=$?\nif test \"$result\" -eq 1; then printf '%s\\n' recovered; else printf wrong; fi\nset +x\nexit \"$result\"\n";
    let expected = Outcome {
        stdout: "recovered\n".into(),
        stderr: "+ false\n+ result=1\n+ test 1 -eq 1\n+ printf '%s\\n' recovered\n+ set +x\n"
            .into(),
        status: 1,
        files: c.files.clone(),
        error: None,
    };
    for slice in [1, 7, 64] {
        assert_eq!(sliced(source, &c, slice), expected);
    }
}
#[test]
fn trace_distinguishes_argument_boundaries_and_c_locale_quoted_bytes() {
    let source = "set -x\nprintf '<%s>\\n' \"$value\"\nset +x\n";
    for (input, shown) in [
        ("North Annex", "'North Annex'"),
        ("", "''"),
        ("can't", "'can'\\''t'"),
        ("'", "\\'"),
        ("a\\b", "'a\\b'"),
        ("a\nb", "$'a\\nb'"),
        ("a\tb", "$'a\\tb'"),
        ("café", "$'caf\\303\\251'"),
        ("a#b", "a#b"),
        ("#a", "'#a'"),
        ("a~b", "a~b"),
        ("~a", "'~a'"),
    ] {
        let mut c = case();
        c.env.insert("value".into(), input.into());
        let r = shell::run(source, &c);
        assert_eq!(r.error, None, "{input:?}");
        assert_eq!(r.stdout, format!("<{input}>\n"));
        assert_eq!(r.stderr, format!("+ printf '<%s>\\n' {shown}\n+ set +x\n"));
        assert_eq!(r.files, c.files);
    }
    let mut c = case();
    c.env.insert("value".into(), "North Annex".into());
    let r = shell::run(&source.replace("\"$value\"", "$value"), &c);
    assert_eq!(r.stdout, "<North>\n<Annex>\n");
    assert_eq!(r.stderr, "+ printf '<%s>\\n' North Annex\n+ set +x\n");
}
#[test]
fn command_redirections_do_not_redirect_the_shell_trace_descriptor() {
    let c = case();
    let r = shell::run(
        "set -x\nprintf '%s\\n' saved > result 2> trace\nset +x\n",
        &c,
    );
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "");
    assert_eq!(r.stderr, "+ printf '%s\\n' saved\n+ set +x\n");
    assert_eq!(
        r.files,
        BTreeMap::from([
            ("/work/keep".into(), "retained\n".into()),
            ("/work/result".into(), "saved\n".into()),
            ("/work/trace".into(), String::new())
        ])
    );
}
#[test]
fn scalar_assignments_show_evaluated_values_without_changing_status_expansion() {
    let c = case();
    let r = shell::run(
        "set -x\nfalse\na=\"$?\" b=\"$a\"\nprintf '%s|%s\\n' \"$a\" \"$b\"\nset +x\n",
        &c,
    );
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "1|1\n");
    assert_eq!(
        r.stderr,
        "+ false\n+ a=1\n+ b=1\n+ printf '%s|%s\\n' 1 1\n+ set +x\n"
    );
}
#[test]
fn empty_assignments_and_tilde_context_match_observed_trace_boundaries() {
    let c = case();
    let r = shell::run(
        "set -x\na='' b=''\nprintf '<%s>|<%s>\\n' \"$a\" \"$b\"\nprintf '%s\\n' 'a:~b' 'a=b~'\nset +x\n",
        &c,
    );
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "<>|<>\na:~b\na=b~\n");
    assert_eq!(
        r.stderr,
        "+ a=\n+ b=\n+ printf '<%s>|<%s>\\n' '' ''\n+ printf '%s\\n' 'a:~b' a=b~\n+ set +x\n"
    );
}
#[test]
fn long_option_and_repeated_toggles_are_real_state_not_command_matching() {
    let c = case();
    let r = shell::run(
        "set +x\nset -o xtrace\nset -x\n: ignored\nset +o xtrace\nprintf done\n",
        &c,
    );
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "done");
    assert_eq!(r.stderr, "+ set -x\n+ : ignored\n+ set +o xtrace\n");
}
#[test]
fn unsupported_compound_tracing_is_explicit_and_nontracing_programs_still_work() {
    for body in [
        "printf a | cat",
        "for x in a; do printf '%s' \"$x\"; done",
        "while false; do :; done",
        "case a in a) printf a ;; esac",
        "f() { printf a; }; f",
        "a=(a); printf '%s' \"${a[0]}\"",
        "a=$(printf a); printf '%s' \"$a\"",
        "IFS= read -r value",
    ] {
        let c = case();
        let ordinary = shell::run(body, &c);
        assert!(ordinary.error.is_none(), "{body}: {:?}", ordinary.error);
        let r = shell::run(&format!("set -x\n{body}"), &c);
        let e = r.error.as_deref().unwrap();
        assert!(
            e.contains("Unsupported xtrace program shape"),
            "{body}: {e}"
        );
        assert_eq!(r.files, c.files);
    }
}
#[test]
fn custom_trace_parameters_and_unimplemented_set_options_fail_honestly() {
    let c = case();
    for source in [
        "PS4='custom'\nset -x",
        "set -v",
        "set -eux",
        "set -o verbose",
        "set -x extra",
    ] {
        let r = shell::run(source, &c);
        assert!(
            r.error.as_ref().is_some_and(|e| e.contains("Unsupported")),
            "{source}: {r:?}"
        );
        assert_eq!(r.files, c.files);
    }
}
#[test]
fn trace_size_uses_actual_quoted_length_and_shared_bounded_output() {
    let mut c = case();
    c.env.insert("value".into(), "x".repeat(9000));
    let r = shell::run("set -x\nprintf '%s\\n' \"$value\"\nset +x", &c);
    assert_eq!(r.error, None);
    assert_eq!(r.stdout.len(), 9001);
    assert!(r.stderr.len() > 9000 && r.stderr.len() < 10000);
    c.env.insert("value".into(), "x".repeat(20000));
    let r = shell::run("set -x\ntest \"$value\" = \"$value\"", &c);
    assert!(r.error.is_some_and(|e| e.contains("limit")));
    assert_eq!(r.files, c.files);
    assert!(r.stdout.len() + r.stderr.len() <= 32768);
}
#[test]
fn repeated_trace_output_and_stop_cannot_freeze_or_earn_credit() {
    let mut c = case();
    c.env.insert("value".into(), "x".repeat(1000));
    let source = format!("set -x\n{}", "printf '%s\\n' \"$value\"\n".repeat(100));
    let r = sliced(&source, &c, 1);
    assert!(r.error.as_ref().is_some_and(|e| e.contains("limit")));
    assert!(r.stdout.len() + r.stderr.len() <= 32768);
    assert_eq!(r.files, c.files);
    let mut vm = Vm::new(&source, &c).unwrap();
    vm.tick(1);
    vm.stop();
    assert!(vm.done());
    assert!(vm.outcome.error.unwrap().contains("Stopped"));
    assert_eq!(vm.outcome.files, c.files);
}
