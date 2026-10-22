use dlms_learning_lab::shell::{self, Case, Outcome, Vm};
use std::collections::BTreeMap;
fn case() -> Case {
    Case {
        name: "Integer arithmetic".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::new(),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn run(source: &str) -> Outcome {
    shell::run(source, &case())
}
fn expect(source: &str, text: &str) {
    let r = run(source);
    assert_eq!(r.error, None, "{source}: {:?}", r.error);
    assert_eq!(r.stdout, text);
    assert_eq!(r.stderr, "");
    assert_eq!(r.status, 0);
}
#[test]
fn arithmetic_obeys_precedence_grouping_signs_and_integer_division() {
    for (expr, value) in [
        ("2+3*4", "14"),
        ("(2+3)*4", "20"),
        ("20/3/2", "3"),
        ("8-3-2", "3"),
        ("-7/3", "-2"),
        ("7/-3", "-2"),
        ("-7%3", "-1"),
        ("7%-3", "1"),
        ("+ - 2 * 3", "-6"),
        ("", "0"),
        ("  ", "0"),
    ] {
        expect(
            &format!("printf '%s\\n' \"$(({expr}))\""),
            &format!("{value}\n"),
        );
    }
    expect(
        "printf '%s:%s\\n' \"pre$((2+3))post\" $((2+3))",
        "pre5post:5\n",
    );
    expect(
        "printf '%s\\n' '$((2+3))' \"\\$((2+3))\"",
        "$((2+3))\n$((2+3))\n",
    );
    expect("printf '%s\\n' \"$((1+$((2*3))))\"", "7\n");
}
#[test]
fn supported_literal_bases_and_fixed_width_wrapping_are_explicit() {
    for (expr, value) in [
        ("010+1", "9"),
        ("0x10+0Xf", "31"),
        ("9223372036854775807+1", "-9223372036854775808"),
        ("0xffffffffffffffff", "-1"),
        ("18446744073709551615", "-1"),
        ("-9223372036854775808 - 1", "9223372036854775807"),
        ("9223372036854775807*2", "-2"),
    ] {
        expect(
            &format!("printf '%s\\n' \"$(({expr}))\""),
            &format!("{value}\n"),
        );
    }
    for source in [
        "printf '%s' \"$((08))\"",
        "printf '%s' \"$((2#10))\"",
        "printf '%s' \"$((18446744073709551616))\"",
    ] {
        assert!(run(source).error.unwrap().contains("Unsupported"));
    }
}
#[test]
fn names_parameters_nested_expansion_and_visible_function_scopes_match_the_subset() {
    let mut c = case();
    c.args = vec!["5".into(), "3".into()];
    c.env.insert("extra".into(), " -2 ".into());
    let r = shell::run(
        "printf '%s\\n' \"$(( $1 * $2 + extra + $# ))\"\nfalse\nprintf '%s\\n' \"$(( $? + 1 ))\"",
        &c,
    );
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "15\n2\n");
    expect(
        "total=0\nfor item in 2 3 4; do total=$((total+item)); done\nprintf '%s\\n' \"$total\"",
        "9\n",
    );
    expect(
        "counter=9\nf() { local counter=2; counter=$((counter+1)); printf '%s\\n' \"$counter\"; }\nf\nprintf '%s\\n' \"$counter\"",
        "3\n9\n",
    );
    expect("empty=''\nprintf '%s\\n' \"$((missing+empty))\"", "0\n");
    expect("part='1 + 2'\nprintf '%s\\n' \"$(( $part * 3 ))\"", "7\n");
    assert!(
        run("part='1 + 2'\nprintf '%s' \"$((part))\"")
            .error
            .unwrap()
            .contains("Unsupported arithmetic variable value")
    );
    assert!(
        run("printf '%s' \"$((UID))\"")
            .error
            .unwrap()
            .contains("Unsupported special")
    );
}
#[test]
fn ordinary_words_and_local_values_expand_before_redirects_while_assignments_are_sequential() {
    let r = run("n=3\nn=$((n+1)) m=$((n+1)) > \"$n\"\nprintf '%s:%s\\n' \"$n\" \"$m\"");
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "4:5\n");
    assert_eq!(r.files.get("/work/4"), Some(&String::new()));
    let r = run("n=3\nn=4 < missing\nprintf '%s\\n' \"$n\"");
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "4\n");
    assert!(r.stderr.contains("No such file"));
    for source in [
        "printf '%s' \"$((1/0))\" > out",
        "n=$((1/0)) > out",
        "f() { local n=$((1/0)) > out; }; f",
    ] {
        let r = run(source);
        assert!(
            r.error
                .as_ref()
                .is_some_and(|e| e.contains("division by zero")),
            "{r:?}"
        );
        assert!(r.files.is_empty());
        assert!(r.stdout.is_empty());
    }
    expect(
        "n=2\nf() { local n=$((n+1)) other=$((n+2)); printf '%s:%s\\n' \"$n\" \"$other\"; }\nf",
        "3:4\n",
    );
}
#[test]
fn malformed_and_out_of_subset_expressions_never_execute_a_command_or_escape() {
    for expr in ["1/0", "1%0", "1 +", "1 2", "(1+2", "1+*2"] {
        let r = run(&format!("printf '%s' \"$(({expr}))\""));
        assert!(r.error.is_some(), "{expr}: {r:?}");
        assert!(r.stdout.is_empty());
    }
    for expr in [
        "a++",
        "++a",
        "1**2",
        "1<<2",
        "1<2",
        "1==2",
        "a=3",
        "1,2",
        "1?2:3",
        "a[1]",
        "!1",
        "~1",
        "1&2",
        "$(touch /tmp/escape)",
        "`touch /tmp/escape`",
        "\"2\"",
        "1.5+2",
        "-9223372036854775808/-1",
    ] {
        let r = run(&format!("printf '%s' $(({expr}))"));
        assert!(
            r.error
                .as_ref()
                .is_some_and(|e| e.to_ascii_lowercase().contains("unsupported")),
            "{expr}: {r:?}"
        );
        assert!(r.files.is_empty());
        assert!(r.stdout.is_empty());
    }
}
#[test]
fn expression_token_nesting_memory_loop_and_stop_bounds_remain_cooperative() {
    let long = "1 ".repeat(2049);
    assert!(
        run(&format!("printf '%s' \"$(({long}))\""))
            .error
            .unwrap()
            .contains("4096-byte limit")
    );
    let tokens = (0..130).map(|_| "1").collect::<Vec<_>>().join("+");
    assert!(
        run(&format!("printf '%s' \"$(({tokens}))\""))
            .error
            .unwrap()
            .contains("token limit")
    );
    let deep = format!("{}1{}", "(".repeat(26), ")".repeat(26));
    assert!(
        run(&format!("printf '%s' \"$(({deep}))\""))
            .error
            .unwrap()
            .contains("nesting limit")
    );
    let mut vm = Vm::new(
        "counter=0\nwhile true; do counter=$((counter+1)); done",
        &case(),
    )
    .unwrap();
    vm.tick(10);
    assert!(!vm.done());
    vm.stop();
    assert!(vm.outcome.error.unwrap().contains("Stopped"));
    assert!(
        run("counter=0\nwhile true; do counter=$((counter+1)); done")
            .error
            .unwrap()
            .contains("2,000 instructions")
    );
}

#[test]
fn pipeline_context_checks_do_not_evaluate_later_assignment_values_prematurely() {
    let r = run("n=0\nn=2 m=$((8/n)) > out | sort\nprintf 'n:%s\\n' \"$n\"");
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "n:0\n");
    assert_eq!(r.files.get("/work/out"), Some(&String::new()));
    expect(
        "unused() { :; }\nn=0\nn=2 m=$((8/n))\nprintf '%s:%s\\n' \"$n\" \"$m\"",
        "2:4\n",
    );
    let r = run("f() { :; }\nempty=''\n$empty f | sort");
    assert!(r.error.unwrap().contains("Unsupported function call"));
}

#[test]
fn multipart_assignment_heads_do_not_split_or_glob_during_function_and_pipeline_checks() {
    expect(
        "unused() { :; }\nvalue='a * b'\nn=$value\"\"\nprintf '<%s>\\n' \"$n\"",
        "<a * b>\n",
    );
    let r = run("value='a * b'\nn=outer\nn=$value\"\" > out | sort\nprintf '<%s>\\n' \"$n\"");
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "<outer>\n");
    assert_eq!(r.files.get("/work/out"), Some(&String::new()));
}
