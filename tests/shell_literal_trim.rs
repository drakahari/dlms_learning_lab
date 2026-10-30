//! Conditional capability vectors, not captured Bash results or authored labs.
use dlms_learning_lab::shell::{self, Case, Outcome, Vm};
use std::collections::BTreeMap;

fn case() -> Case {
    let files = BTreeMap::from([("/work/keep.txt".into(), "retained\n".into())]);
    Case {
        name: "Literal-removal capability".into(),
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
fn sliced(source: &str, c: &Case, slice: usize) -> Outcome {
    let mut vm = Vm::new(source, c).expect("Supported fixed program compiles");
    for _ in 0..20_000 {
        if vm.done() {
            return vm.outcome;
        }
        vm.tick(slice);
    }
    panic!("Fixed program exceeded the finite cooperative test bound");
}
fn expect(source: &str, c: &Case, stdout: &str, files: BTreeMap<String, String>) {
    for slice in [1, 7, 64] {
        assert_eq!(
            sliced(source, c, slice),
            Outcome {
                stdout: stdout.into(),
                stderr: String::new(),
                status: 0,
                files: files.clone(),
                error: None,
            },
            "slice={slice}, {source}"
        );
    }
}

#[test]
fn scalar_truth_table_removes_only_one_literal_edge_without_mutating_values() {
    assert_eq!(shell::EVALUATOR_REVISION, 15);
    let source = r#"printf '<%s>|<%s>|<%s>\n' "${value#pre}" "${value%post}" "$value""#;
    for (value, text) in [
        (None, "<>|<>|<>\n"),
        (Some(""), "<>|<>|<>\n"),
        (Some("preXpost"), "<Xpost>|<preX>|<preXpost>\n"),
        (
            Some("preprepostpost"),
            "<prepostpost>|<preprepost>|<preprepostpost>\n",
        ),
        (Some("elsewhere"), "<elsewhere>|<elsewhere>|<elsewhere>\n"),
    ] {
        let mut c = case();
        if let Some(value) = value {
            c.env.insert("value".into(), value.into());
        }
        expect(source, &c, text, c.files.clone());
    }
    let mut c = case();
    c.env.insert("value".into(), "é-a#b%c.é".into());
    expect(
        r#"printf '%s|%s|%s\n' "${value#é-}" "${value%.é}" "${value#}""#,
        &c,
        "a#b%c.é|é-a#b%c|é-a#b%c.é\n",
        c.files.clone(),
    );
    c.env.insert("value".into(), "a#b%cX".into());
    expect(r#"printf '%s' "${value#a#b%c}""#, &c, "X", c.files.clone());
}

#[test]
fn function_positional_and_local_bindings_restore_the_callers_values() {
    let mut c = case();
    c.args.push("--outside".into());
    let source = r#"
value=pre-global
strip() {
    local value=pre-local
    printf '%s:%s\n' "${1#--}" "${value#pre-}"
}
strip --inside
printf '%s:%s\n' "${1#--}" "$value"
"#;
    expect(
        source,
        &c,
        "inside:local\noutside:pre-global\n",
        c.files.clone(),
    );
}

#[test]
fn outer_quotes_and_existing_split_glob_rules_preserve_field_boundaries() {
    let mut c = case();
    c.files.insert("/work/b1".into(), "one\n".into());
    c.env.insert("value".into(), "prea b?".into());
    let source = r#"
for word in "${value#pre}"; do printf '<%s>\n' "$word"; done
for word in ${value#pre}; do printf '<%s>\n' "$word"; done
count() { printf '%s\n' "$#"; }
count "${missing#pre}"
count ${missing#pre}
printf '%s\n' '${value#pre}'
"#;
    expect(
        source,
        &c,
        "<a b?>\n<a>\n<b1>\n1\n0\n${value#pre}\n",
        c.files.clone(),
    );
}

#[test]
fn hostile_value_data_is_not_reparsed_and_utf8_results_are_whole_text() {
    let mut c = case();
    c.env
        .insert("value".into(), "pre$(printf NEVER);*?.log".into());
    expect(
        r#"printf '%s\n%s\n' "${value#pre}" "${value%.log}""#,
        &c,
        "$(printf NEVER);*?.log\npre$(printf NEVER);*?\n",
        c.files.clone(),
    );
}

#[test]
fn redirects_captures_and_arithmetic_use_the_existing_preparation_path() {
    let c = case();
    let source = r#"
name=preout.txt
printf 'saved\n' > "${name#pre}"
n=pre7
printf '%s\n' "$(( ${n#pre} + 1 ))"
readback=$(cat out.txt)
printf '%s\n' "$readback"
"#;
    let mut files = c.files.clone();
    files.insert("/work/out.txt".into(), "saved\n".into());
    expect(source, &c, "8\nsaved\n", files);
}

#[test]
fn older_default_array_count_and_plain_hash_dispatch_remain_unchanged() {
    let mut c = case();
    c.args = vec!["one".into(), "two".into()];
    expect(
        r#"items=(first second); printf '%s|%s|%s|%s\n' "${missing:-pre#tail}" "${missing:-post%tail}" "${#items[@]}" "$#""#,
        &c,
        "pre#tail|post%tail|2|2\n",
        c.files.clone(),
    );
    expect(
        r#"getopts o: option -o prevalue; printf '<%s>:<%s>\n' "${OPTIND#3}" "${OPTARG#pre}""#,
        &case(),
        "<>:<value>\n",
        case().files,
    );
}

#[test]
fn scheduled_subsets_continue_refusing_parameter_operators_before_execution() {
    let mut c = case();
    c.env.insert("value".into(), "prevalue".into());
    // Cron processes percent escapes before shell quotes. One-off jobs do not.
    // Escape both the printf format and the suffix operator for cron so these
    // negatives reach their parameter guard instead of the percent-input gate.
    for (cron_source, one_off_source) in [
        (
            r#"printf '\%s' "${value#pre}" > keep.txt"#,
            r#"printf '%s' "${value#pre}" > keep.txt"#,
        ),
        (
            r#"printf '\%s' "${value\%value}" > keep.txt"#,
            r#"printf '%s' "${value%value}" > keep.txt"#,
        ),
    ] {
        let cron_result = shell::cron::start(cron_source, &c);
        assert!(cron_result.is_err());
        let cron_error = cron_result.err().unwrap();
        assert!(
            cron_error.contains("Unsupported scheduled parameter"),
            "Cron must reach its parameter guard: {cron_error}"
        );
        let one_off_result = shell::one_off::start(one_off_source, &c);
        assert!(one_off_result.is_err());
        let one_off_error = one_off_result.err().unwrap();
        assert!(
            one_off_error.contains("Unsupported one-off parameter"),
            "One-off must reach its parameter guard: {one_off_error}"
        );
    }
    let cron_source = r#"printf '\%s' "${value}""#;
    let one_off_source = r#"printf '%s' "${value}""#;
    assert_eq!(shell::cron::prepare(cron_source).unwrap(), one_off_source);
    for mut vm in [
        shell::cron::start(cron_source, &c).unwrap(),
        shell::one_off::start(one_off_source, &c).unwrap(),
    ] {
        for _ in 0..100 {
            if vm.done() {
                break;
            }
            vm.tick(1);
        }
        assert!(vm.done());
        assert_eq!(
            vm.outcome,
            Outcome {
                stdout: "prevalue".into(),
                stderr: String::new(),
                status: 0,
                files: c.files.clone(),
                error: None,
            }
        );
    }
}

#[test]
fn unsupported_new_grammar_is_rejected_before_any_earlier_command_or_redirect() {
    for key in [
        "value##pre",
        "value%%post",
        "value#*",
        "value%?",
        "value#[ab]",
        "value#~home",
        "value#$other",
        "value#${other}",
        "value#$(printf bad)",
        "value#'inner'",
        "value#\"inner\"",
        "value#x\\y",
        "value#x`y`",
        "value#x\ny",
        "value#x\ty",
        "value#(x)",
        "0#x",
        "10%x",
        "UID#x",
        "IFS%x",
        "BASH_FORBIDDEN#x",
        "items[0]#x",
        "@#x",
        "*%x",
        "?#x",
    ] {
        let source = format!(
            "printf earlier; printf changed > keep.txt; if false; then printf '%s' \"${{{key}}}\"; fi"
        );
        let c = case();
        let error = Vm::new(&source, &c)
            .err()
            .expect("Grammar must fail in compile preflight");
        assert!(error.contains("Unsupported"), "{key}: {error}");
        let out = shell::run(&source, &c);
        assert!(
            out.error
                .as_ref()
                .is_some_and(|e| e.contains("Unsupported"))
        );
        assert_eq!(
            (out.stdout.as_str(), out.stderr.as_str(), out.status),
            ("", "", 2)
        );
        assert_eq!(out.files, c.files);
    }
    let c = case();
    let out = shell::run("printf changed > keep.txt; printf '%s' \"${value#pre\"", &c);
    assert!(out.error.is_some());
    assert!(out.stdout.is_empty() && out.stderr.is_empty());
    assert_eq!(out.status, 2);
    assert_eq!(out.files, c.files);
}

#[test]
fn array_type_failure_is_runtime_and_does_not_run_its_own_redirect() {
    let c = case();
    let source = r#"printf earlier; value=(pretail other); printf '%s' "${value#pre}" > keep.txt"#;
    assert!(
        Vm::new(source, &c).is_ok(),
        "A name's later array type is not lexical grammar"
    );
    let out = shell::run(source, &c);
    assert!(
        out.error
            .as_ref()
            .is_some_and(|e| e.contains("indexed array"))
    );
    assert_eq!(
        (out.stdout.as_str(), out.stderr.as_str(), out.status),
        ("earlier", "", 2)
    );
    assert_eq!(out.files, c.files);
}

#[test]
fn repeated_trim_uses_one_expansion_work_budget_across_loop_function_and_capture() {
    let mut c = case();
    c.env.insert("value".into(), "short".into());
    let words = "item ".repeat(120);
    let pattern = "x".repeat(9000);
    let assignment = format!("result=\"${{value#{pattern}}}\"");
    let programs = [
        format!("for item in {words}; do {assignment}; done"),
        format!("trim() {{ {assignment}; }}; for item in {words}; do trim; done"),
        format!("for item in {words}; do result=$(printf '%s' \"${{value#{pattern}}}\"); done"),
    ];
    for source in programs {
        assert!(source.len() < shell::MAX_SOURCE);
        let out = shell::run(&source, &c);
        assert!(
            out.error
                .as_ref()
                .is_some_and(|e| e.contains("Expansion work limit")),
            "{out:?}"
        );
        assert_eq!(
            (out.stdout.as_str(), out.stderr.as_str(), out.status),
            ("", "", 2)
        );
        assert_eq!(out.files, c.files);
    }
}

#[test]
fn existing_argument_memory_and_output_limits_admit_no_file_publication() {
    let mut c = case();
    c.env
        .insert("large".into(), format!("pre{}", "v".repeat(90_000)));
    let out = shell::run(
        r#"printf '%s' "${large#pre}" "${large#pre}" "${large#pre}" > keep.txt"#,
        &c,
    );
    assert!(
        out.error
            .as_ref()
            .is_some_and(|e| e.to_lowercase().contains("memory")),
        "{out:?}"
    );
    assert_eq!(out.files, c.files);
    let mut c = case();
    c.env
        .insert("many".into(), format!("pre{}", "x ".repeat(200)));
    let out = shell::run("printf '%s' ${many#pre} > keep.txt", &c);
    assert!(
        out.error
            .as_ref()
            .is_some_and(|e| e.to_lowercase().contains("argument")),
        "{out:?}"
    );
    assert_eq!(out.files, c.files);
    c.env
        .insert("value".into(), format!("pre{}", "v".repeat(512)));
    let out = shell::run("while true; do printf '%s' \"${value#pre}\"; done", &c);
    assert!(
        out.error
            .as_ref()
            .is_some_and(|e| e.to_lowercase().contains("output")),
        "{out:?}"
    );
    assert!(out.stdout.len() <= 32_768);
    assert_eq!(out.files, c.files);
}
